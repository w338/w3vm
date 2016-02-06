use std::fmt::{Debug};

#[derive(Debug, PartialEq)]
enum Component<I, O> where O: Clone + Debug, I: PartialEq + Debug {
    Output(O),
    Input(I, usize),
    Jump(usize)
}

#[derive(Debug)]
struct SliceDFA<'a, I, O> where I: 'a + PartialEq + Debug, O: 'a + Clone + Debug {
    components: &'a [Component<I, O>]
}

impl<'a, I, O> SliceDFA<'a, I, O> where I: 'a + PartialEq + Debug, O: 'a + Clone + Debug {
    fn new(components: &'a [Component<I, O>]) -> Self {
        SliceDFA {
            components: components
        }
    }

    fn eval<It>(&self, state: usize, input: It, output: &mut [O])
        -> (usize, usize) where It: Iterator<Item=&'a I> {
        let mut input = input;
        let mut outc = 0;
        let mut state = state;
        let mut next_input = input.next();
        let mut states_since_input = 0;
        loop {
            let component = &self.components[state];
            match component {
                &Component::Output(ref o) => {
                    if outc >= output.len() {
                        return (outc, state);
                    }
                    output[outc] = o.clone();
                    outc += 1;
                    state += 1;
                    states_since_input += 1;
                },
                &Component::Input(ref i, next_state) => {
                    states_since_input = 0;
                    match next_input {
                        Some(to_cmp) => {
                            if i == to_cmp {
                                state = next_state;
                                next_input = input.next();
                            } else {
                                state += 1;
                            }
                        },
                        None => {
                            return (outc, state);
                        }
                    }
                },
                &Component::Jump(next_state) => {
                    states_since_input += 1;
                    if states_since_input > self.components.len() {
                        return (outc, state);
                    }
                    state = next_state;
                }
            }
        }
    }
}

#[test]
fn it_inverts() {
    let input: &[bool] = &[true, true, false, true][..];
    let dfa = [
        Component::Input(true, 2),
        Component::Input(false, 4),
        Component::Output(false),
        Component::Jump(0),
        Component::Output(true),
        Component::Jump(0)
    ];
    let mut output = [false, false, false, false];
    assert_eq!(SliceDFA::new(&dfa[..]).eval(0, input.iter(), &mut output), (4, 0));
    assert_eq!(output, [false, false, true, false]);
}

#[test]
fn it_stops_eventually() {
    let input: &[bool] = &[][..];
    let dfa = [
        Component::Jump(0),
        Component::Input(false, 0),
        Component::Output(false),
    ];
    let mut output = [];
    assert_eq!(SliceDFA::new(&dfa[..]).eval(0, input.iter(), &mut output), (0, 0));
}

#[test]
fn it_doesnt_panic_on_output() {
    let input: &[bool] = &[][..];
    let dfa = [
        Component::Output(false),
        Component::Jump(0),
        Component::Input(false, 0),
    ];
    let mut output = [false];
    assert_eq!(SliceDFA::new(&dfa[..]).eval(0, input.iter(), &mut output), (1, 0));
}
