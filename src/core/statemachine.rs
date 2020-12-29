// TODO:
// - Error handling!
// - Checkpoint & trim

use serde::{Deserialize, Serialize};

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
}

#[derive(Deserialize, Serialize)]
pub enum LogEntry<S, D>{
    Checkpoint(S),
    Delta(D),
}

pub struct Saver<S: serde::Serialize + Clone, D: serde::Serialize + Clone> {
    sink: std::rc::Rc<std::sync::Mutex<dyn std::io::Write>>,
    // https://doc.rust-lang.org/std/marker/struct.PhantomData.html#examples
    phantom_s: std::marker::PhantomData<S>,
    phantom_d: std::marker::PhantomData<D>,
}

impl <S: serde::Serialize + Clone, D: serde::Serialize + Clone> Saver<S, D> {
    pub fn new(sink: std::rc::Rc<std::sync::Mutex<dyn std::io::Write>>) -> Saver<S, D> {
        return Saver{
            sink: sink,
            phantom_s: std::marker::PhantomData,
            phantom_d: std::marker::PhantomData,
        };
    }

    pub fn append_checkpoint(&mut self, checkpoint: &S) {
        let e = LogEntry::<S, D>::Checkpoint(checkpoint.clone());
        let as_json = serde_json::to_string(&e).expect("XXX");
        let mut sink = self.sink.lock().expect("rc::get_mut");
        sink.write(as_json.as_bytes()).expect("xxx");
        sink.write("\n".as_bytes()).expect("xxx");

    }

    pub fn append_delta(&mut self, delta: &D) {
        let e = LogEntry::<S, D>::Delta(delta.clone());
        let as_json = serde_json::to_string(&e).expect("XXX");
        let mut sink = self.sink.lock().expect("rc::get_mut");
        sink.write(as_json.as_bytes()).expect("xxx");
        sink.write("\n".as_bytes()).expect("xxx");
    }


}

pub struct PersistentStateMachine<S: serde::de::DeserializeOwned + serde::Serialize + Clone, D: serde::de::DeserializeOwned + serde::Serialize + Clone> {
    machine: StateMachine<S, D>,
    saver: Saver<S, D>,
}

impl <S: serde::de::DeserializeOwned + serde::Serialize + Clone, D: serde::de::DeserializeOwned + serde::Serialize + Clone> PersistentStateMachine<S, D> {
    pub fn init(initial_state: S,
                apply_fn: Box<dyn Fn(&mut S, &D)>,
                mut saver: Saver<S, D>) -> PersistentStateMachine<S, D> {
        saver.append_checkpoint(&initial_state);
        return PersistentStateMachine{
            machine: StateMachine::new(initial_state, apply_fn),
            saver: saver,
        };
    }

    pub fn recover(lines: &mut dyn Iterator<Item=String>,
                   apply_fn: &dyn Fn(&mut S, &D)) -> S {
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
}

#[cfg(test)]
mod statemachine_tests {
    use super::PersistentStateMachine;
    use super::Saver;

    use crate::serde::{Deserialize, Serialize};

    #[derive(Deserialize, Serialize, Copy, Clone)]
    struct Total {v: i32}

    #[derive(Deserialize, Serialize, Copy, Clone)]
    struct Increment{i: i32}

    #[test]
    fn simple() {
        let logfile: std::rc::Rc<std::sync::Mutex<Vec<u8>>> = std::rc::Rc::new(std::sync::Mutex::new(vec![]));

        let apply_fn =
            |state: &mut Total, delta: &Increment| state.v += delta.i;

        {
            let saver = Saver::<Total, Increment>{
                sink: logfile.clone(),
                phantom_d: std::marker::PhantomData,
                phantom_s: std::marker::PhantomData,
            };

            let mut psm = PersistentStateMachine::init(
                Total{v: 0},
                Box::new(apply_fn),
                saver);

            assert_eq!(0, psm.state().v, "Initial state check");

            psm.apply(&Increment{i: 1});
            assert_eq!(1, psm.state().v, "+1 state check");

            psm.apply(&Increment{i: 10});
            assert_eq!(11, psm.state().v, "+10 state check");
        }

        use std::io::BufRead;

        let state = PersistentStateMachine::recover(
            &mut logfile.lock().unwrap().lines().map(|res| res.unwrap()), &apply_fn);
        assert_eq!(11, state.v);
    }
}
