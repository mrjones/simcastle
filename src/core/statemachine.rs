use serde::{Deserialize, Serialize};
//use serde::de::DeserializeOwned;

pub struct StateMachine<S, D> {
    state: S,
    apply_fn: Box<dyn Fn(&mut S, &D)>,
}

impl <S, D> StateMachine<S, D> {
    pub fn new(initial_state: S, apply_fn: Box<dyn Fn(&mut S, &D)>) -> StateMachine<S, D> {
        return StateMachine{state: initial_state, apply_fn: apply_fn};
    }

    pub fn apply(&mut self, delta: &D) {
        (*self.apply_fn)(&mut self.state, delta);
    }

    pub fn state(&self) -> &S {
        return &self.state;
    }

    pub fn unsafe_mutable_state(&mut self) -> &mut S {
        return &mut self.state;
    }
}

#[derive(Deserialize, Serialize)]
pub enum LogEntry<S, D>{
    Checkpoint(S),
    Delta(D),
}

pub struct Saver<'a, S: serde::Serialize, D: serde::Serialize> {
    sink: &'a mut dyn std::io::Write,
    // https://doc.rust-lang.org/std/marker/struct.PhantomData.html#examples
    phantom_s: std::marker::PhantomData<S>,
    phantom_d: std::marker::PhantomData<D>,
}

impl <'a, S: serde::Serialize + Copy, D: serde::Serialize + Copy> Saver<'a, S, D> {
    pub fn append_checkpoint(&mut self, checkpoint: &S) {
        let e = LogEntry::<S, D>::Checkpoint(*checkpoint);
        let as_json = serde_json::to_string(&e).expect("XXX");
        self.sink.write(as_json.as_bytes()).expect("xxx");
        self.sink.write("\n".as_bytes()).expect("xxx");

    }

    pub fn append_delta(&mut self, delta: &D) {
        let e = LogEntry::<S, D>::Delta(*delta);
        let as_json = serde_json::to_string(&e).expect("XXX");
        self.sink.write(as_json.as_bytes()).expect("xxx");
        self.sink.write("\n".as_bytes()).expect("xxx");
    }
}

pub struct PersistentStateMachine<'io, S: serde::de::DeserializeOwned + serde::Serialize, D: serde::de::DeserializeOwned + serde::Serialize> {
    machine: StateMachine<S, D>,
    saver: &'io mut Saver<'io, S, D>,
}

impl <'io, S: serde::de::DeserializeOwned + serde::Serialize + Copy, D: serde::de::DeserializeOwned + serde::Serialize + Copy> PersistentStateMachine<'io, S, D> {
    pub fn new(initial_state: S, apply_fn: Box<dyn Fn(&mut S, &D)>, saver: &'io mut Saver<'io, S, D>) -> PersistentStateMachine<S, D> {
        saver.append_checkpoint(&initial_state);
        return PersistentStateMachine{
            machine: StateMachine::new(initial_state, apply_fn),
            saver: saver,
        };
    }

    pub fn recover<'a, >(lines: &mut dyn Iterator<Item=String>, apply_fn: &dyn Fn(&mut S, &D)) -> S {
        let head = lines.next().expect("recover: no head");
        let initial_entry: LogEntry<S, D> = serde_json::from_str(&head).expect("recover: couldn't parse initial CP");

        let mut state = match initial_entry {
            LogEntry::Checkpoint(cp) => cp,
            LogEntry::Delta(_) => panic!("log started with delta"),
        };

        for entry in lines {
            let entry_struct: LogEntry<S, D> = serde_json::from_str(&entry).expect("recover: couldn't parse additional line");
            match entry_struct {
                LogEntry::Checkpoint(cp) => state = cp,
                LogEntry::Delta(d) => (*apply_fn)(&mut state, &d),
            }
        }

        return state;
    }

    pub fn apply(&mut self, delta: &D) {
        self.saver.append_delta(delta);
        self.machine.apply(delta);
    }

    pub fn state(&self) -> &S {
        return self.machine.state();
    }

    pub fn unsafe_mutable_state(&mut self) -> &mut S {
        return self.machine.unsafe_mutable_state();
    }
}


#[cfg(test)]
mod statemachine_tests {
    use super::LogEntry;
    use super::PersistentStateMachine;
    use super::Saver;

    use crate::serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Copy, Clone)]
    struct Total {v: i32}

    #[derive(Deserialize, Serialize, Copy, Clone)]
    struct Increment{i: i32}

    #[test]
    fn simple() {
        let mut logfile: Vec<u8> = vec![];

        let apply_fn =
            |state: &mut Total, delta: &Increment| state.v += delta.i;

        {
            let mut saver = Saver::<Total, Increment>{
                sink: &mut logfile,
                phantom_d: std::marker::PhantomData,
                phantom_s: std::marker::PhantomData,
            };

            let mut psm = PersistentStateMachine::new(
                Total{v: 0},
                Box::new(apply_fn),
                &mut saver);

            assert_eq!(0, psm.state().v);

            psm.apply(&Increment{i: 1});
            assert_eq!(1, psm.state().v);

            psm.apply(&Increment{i: 10});
            assert_eq!(11, psm.state().v);
        }

        use std::io::BufRead;

        let state = PersistentStateMachine::recover(
            &mut logfile.lines().map(|res| res.unwrap()), &apply_fn);
        assert_eq!(11, state.v);
    }
}
