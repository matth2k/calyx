#[allow(unused)]
use crate::port_bindings;
#[allow(unused)]
use crate::primitives::{combinational as comb, stateful as stfl, Primitive};
#[allow(unused)]
use crate::values::{OutputValue, ReadableValue, TickableValue, Value};
#[allow(unused)]
use calyx::ir;

#[test]
fn mult_flickering_go() {
    let mut mult = stfl::StdMultPipe::from_constants(32);
    port_bindings![binds;
        go -> (0, 1),
        left -> (2, 32),
        right -> (7, 32)
    ];
    mult.validate_and_execute(&binds);
    port_bindings![binds;
        go -> (1, 1),
        left -> (3, 32),
        right -> (7, 32)
    ];
    mult.validate_and_execute(&binds);
    mult.do_tick();
    mult.do_tick();
    let mut output_vals = mult.do_tick().into_iter(); //should output done and 21, not 14
    assert_eq!(output_vals.len(), 2);
    let out = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(out.as_u64(), 21);
    let done = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(done.as_u64(), 1);
    output_vals = mult.do_tick().into_iter();
    assert_eq!(output_vals.len(), 0);
}

#[test]
fn test_std_mult_pipe() {
    let mut mult = stfl::StdMultPipe::from_constants(32);
    port_bindings![binds;
        go -> (1, 1),
        left -> (2, 32),
        right -> (7, 32)
    ];
    //each execute needs to be followed by a do_tick() for the input to be
    //captured
    mult.validate_and_execute(&binds);
    let output_vals = mult.do_tick(); //internal q: [14, N]
    assert_eq!(output_vals.len(), 0);
    port_bindings![binds;
        go -> (1, 1),
        left -> (3, 32),
        right -> (7, 32)
    ];
    // I don't think that as written this works correctly. If the go "flickers" on
    // for a portion of the cycle but does not remain high (i.e. is not high by
    // the time do_tick is called) then the multiplier should not run.
    // based on the above comment, b/c go is now low, nothing should be written
    // to the queue!
    mult.validate_and_execute(&binds);
    port_bindings![binds;
        go -> (0, 1), //b/c go is low, this should not overwrite 3*7!
        left -> (4, 32),
        right -> (7, 32)
    ];
    mult.validate_and_execute(&binds);
    let output_vals = mult.do_tick(); //internal q: [N, 14]
    assert_eq!(output_vals.len(), 0);
    port_bindings![binds;
        go -> (1, 1),
        left -> (5, 32),
        right -> (7, 32)
    ];
    mult.validate_and_execute(&binds);
    let mut output_vals = mult.do_tick().into_iter(); //should output done and 14, internal queue: [35, N]
    assert_eq!(output_vals.len(), 2);
    let out = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(out.as_u64(), 14);
    let done = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(done.as_u64(), 1);
    //now tick 3 more times; get empty vec, 35, empty vec
    output_vals = mult.do_tick().into_iter(); //should output empty vec
    assert_eq!(output_vals.len(), 0);
    output_vals = mult.do_tick().into_iter(); //should output done and 35
    assert_eq!(output_vals.len(), 2);
    let out = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(out.as_u64(), 35);
    let done = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(done.as_u64(), 1);
    //none (empty output vec)
    output_vals = mult.do_tick().into_iter(); //should output empty vec
    assert_eq!(output_vals.len(), 0);
}

#[test]
fn test_std_div_pipe() {
    let mut div = stfl::StdDivPipe::from_constants(32);
    port_bindings![binds;
        go -> (1, 1),
        left -> (20, 32),
        right -> (7, 32)  //20/7 = 2 r. 6
    ];
    //each execute needs to be followed by a do_tick() for the input to be
    //captured
    div.validate_and_execute(&binds);
    let output_vals = div.do_tick(); //internal q: [(2, 6), N]
    assert_eq!(output_vals.len(), 0);
    port_bindings![binds;
        go -> (1, 1),
        left -> (20, 32),
        right -> (6, 32) //20/6 = 3 r. 2
    ];
    div.validate_and_execute(&binds);
    port_bindings![binds;
        go -> (0, 1), //b/c go is low, this should not overwrite 20/6!
        left -> (4, 32),
        right -> (7, 32)
    ];
    // I don't think that as written this works correctly. If the go "flickers" on
    // for a portion of the cycle but does not remain high (i.e. is not high by
    // the time do_tick is called) then the multiplier should not run.
    // based on the above comment, b/c go is now low, nothing should be written
    // to the queue!
    div.validate_and_execute(&binds);
    let output_vals = div.do_tick(); //internal q: [N, (2, 6)]
    assert_eq!(output_vals.len(), 0);
    port_bindings![binds;
        go -> (1, 1),
        left -> (20, 32),
        right -> (5, 32) //20/5 = 4 r. 0
    ];
    div.validate_and_execute(&binds);
    let mut output_vals = div.do_tick().into_iter(); //should output done and out_quotient 2 and out_remainder 6
                                                     //internal q: [(4, 0), N]
    assert_eq!(output_vals.len(), 3);
    let out_quotient = output_vals.next().unwrap();
    assert_eq!(out_quotient.0, "out_quotient");
    assert_eq!(out_quotient.1.unwrap_imm().as_u64(), 2);
    let out_remainder = output_vals.next().unwrap();
    assert_eq!(out_remainder.0, "out_remainder");
    assert_eq!(out_remainder.1.unwrap_imm().as_u64(), 6);
    let done = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(done.as_u64(), 1);
    //internal q: [(4, 0), N]
    output_vals = div.do_tick().into_iter(); //give none
    assert_eq!(output_vals.len(), 0);
    //internal q: [N, (4, 0)]
    output_vals = div.do_tick().into_iter(); //out_q : 4, out_r: 0
    assert_eq!(output_vals.len(), 3);
    let out_quotient = output_vals.next().unwrap();
    assert_eq!(out_quotient.0, "out_quotient");
    assert_eq!(out_quotient.1.unwrap_imm().as_u64(), 4);
    let out_remainder = output_vals.next().unwrap();
    assert_eq!(out_remainder.0, "out_remainder");
    assert_eq!(out_remainder.1.unwrap_imm().as_u64(), 0);
    //let done = output_vals.next().unwrap().1.unwrap_imm();
    //none (empty output vec)
    output_vals = div.do_tick().into_iter(); //should output done and 14
    assert_eq!(output_vals.len(), 0);
}

#[test]
fn test_std_reg_imval() {
    let mut reg1 = stfl::StdReg::from_constants(6);
    //see that unitialized register, executed w/ write_en low,
    //returns 0, and no DONE
    port_bindings![binds;
        r#in -> (16, 6),
        write_en -> (0, 1)
    ];
    let output_vals = reg1.validate_and_execute(&binds);
    assert_eq!(0, output_vals.len()); //output_vals should be empty from execute
    let output_vals = reg1.do_tick();
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(2, output_vals.len());
    //should be a 0 and a 0 ([out] and [done])
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    let d = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(d.as_u64(), 0);
    //now have write_en high and see output from do_tick() is 16, 1
    port_bindings![binds;
        r#in -> (16, 6),
        write_en -> (1, 1)
    ];
    let output_vals = reg1.validate_and_execute(&binds);
    assert_eq!(0, output_vals.len()); //output_vals should be empty from execute
    let output_vals = reg1.do_tick();
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    //should be a 16 and a 1 ([out] and [done])
    let (out, done_val) =
        (output_vals.next().unwrap(), output_vals.next().unwrap());
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 16);
    let d = done_val.1.unwrap_imm();
    assert_eq!(d.as_u64(), 1);
    //now try to overwrite but w/ write_en low, and see 16 and 0 is returned
    port_bindings![binds;
        r#in -> (16, 6),
        write_en -> (0, 1)
    ];
    let output_vals = reg1.validate_and_execute(&binds);
    assert_eq!(0, output_vals.len()); //output_vals should be empty from execute
    let output_vals = reg1.do_tick();
    ////should be a 16 and a 0 ([out] and [done])
    assert_eq!(2, output_vals.len());
    let mut output_vals = output_vals.into_iter();
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 16);
    let d = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(d.as_u64(), 0);
}

#[test]
fn test_std_mem_d1() {
    let mut mem = stfl::StdMemD1::from_constants(6, 10, 4);
    //see that unitialized mem, executed w/ write_en low,
    //returns 0, and no DONE
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (0, 1),
        addr0 -> (4, 4)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    let output_vals = mem.do_tick(); //this should have low done
    assert_eq!(output_vals.len(), 1);
    let d = output_vals.into_iter().next().unwrap().1.unwrap_imm();
    assert_eq!(d.as_u64(), 0);

    //now have write_en high and see output of execute is 0, and output of write is 16
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (1, 1),
        addr0 -> (4, 4)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
                                      //should be a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    //now that we are ticking, update should be written (and returned)
    let output_vals = mem.do_tick(); //this should have read_data and done, cuz write_en was hgih
    assert_eq!(output_vals.len(), 2);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    let rd = output_vals.next().unwrap();
    let d = output_vals.next().unwrap();
    assert_eq!(rd.1.unwrap_imm().as_u64(), 16);
    assert_eq!(d.1.unwrap_imm().as_u64(), 1);
    //now try to overwrite but w/ write_en low, and see 16 and 0 is returned
    port_bindings![binds;
        write_data -> (3, 6),
        write_en -> (0, 1),
        addr0 -> (4, 4)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    assert_eq!(1, output_vals.len()); //we should get read_data combinationally from [addr0]
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    //should be a 16 and a 0 ([out] and [done])
    assert_eq!(output_vals.len(), 1);
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 16);
    let mut output_vals = mem.do_tick().into_iter();
    let d = output_vals.next().unwrap().1.unwrap_imm();
    assert_eq!(d.as_u64(), 0);
}

#[test]
fn test_std_mem_d2() {
    let mut mem = stfl::StdMemD2::from_constants(6, 4, 4, 2, 2);
    //see that unitialized mem, executed w/ write_en low,
    //returns 0, and no DONE
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (0, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
                                      //should be a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    let output_vals = mem.do_tick(); //this should have low done
    assert_eq!(output_vals.len(), 1);
    let d = output_vals.into_iter().next().unwrap().1.unwrap_imm();
    assert_eq!(d.as_u64(), 0);
    //now have write_en high and see output of execute is 0, and output of write is 16
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (1, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
                                      //should be a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    //now that we are ticking, update should be written (and returned)
    let output_vals = mem.do_tick(); //this should have read_data and done, cuz write_en was hgih
    assert_eq!(output_vals.len(), 2);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    let rd = output_vals.next().unwrap();
    let d = output_vals.next().unwrap();
    assert_eq!(rd.1.unwrap_imm().as_u64(), 16);
    assert_eq!(d.1.unwrap_imm().as_u64(), 1);
    //now try to overwrite but w/ write_en low, and see 16 and 0 is returned
    port_bindings![binds;
        write_data -> (3, 6),
        write_en -> (0, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    assert_eq!(1, output_vals.len()); //we should get read_data combinationally from [addr0]
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 16);
}

#[test]
fn test_std_mem_d3() {
    let mut mem = stfl::StdMemD3::from_constants(6, 4, 4, 4, 2, 2, 2);
    //see that unitialized mem, executed w/ write_en low,
    //returns 0, and no DONE
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (0, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2),
        addr2 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
                                      //should be a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    let output_vals = mem.do_tick(); //this should have done as 0
    assert_eq!(output_vals.len(), 1);
    let d = output_vals.into_iter().next().unwrap().1.unwrap_imm();
    assert_eq!(d.as_u64(), 0);

    //now have write_en high and see output of execute is 0, and output of write is 16
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (1, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2),
        addr2 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
                                      //should be a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    //now that we are ticking, update should be written (and returned)
    let output_vals = mem.do_tick(); //this should have read_data and done, cuz write_en was hgih
    assert_eq!(output_vals.len(), 2);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    let rd = output_vals.next().unwrap();
    let d = output_vals.next().unwrap();
    assert_eq!(rd.1.unwrap_imm().as_u64(), 16);
    assert_eq!(d.1.unwrap_imm().as_u64(), 1);
    //now try to overwrite but w/ write_en low, and see 16 and 0 is returned
    port_bindings![binds;
        write_data -> (3, 6),
        write_en -> (0, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2),
        addr2 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    assert_eq!(1, output_vals.len()); //we should get read_data combinationally from [addr0]
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    //should be a 16 and a 1 ([out] and [done])
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 16);
}

#[test]
fn test_std_mem_d4() {
    let mut mem = stfl::StdMemD4::from_constants(6, 4, 4, 4, 4, 2, 2, 2, 2);
    //see that unitialized mem, executed w/ write_en low,
    //returns 0, and no DONE
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (0, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2),
        addr2 -> (3, 2),
        addr3 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
                                      //should be a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    let output_vals = mem.do_tick(); //this should have low done
    assert_eq!(output_vals.len(), 1);
    let d = output_vals.into_iter().next().unwrap().1.unwrap_imm();
    assert_eq!(d.as_u64(), 0);

    //now have write_en high and see output of execute is 0, and output of write is 16
    port_bindings![binds;
        write_data -> (16, 6),
        write_en -> (1, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2),
        addr2 -> (3, 2),
        addr3 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    assert_eq!(1, output_vals.len()); //should just have data @ addr0, which is a 0
                                      //should be a 0
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 0);
    assert_eq!(out.0, "read_data");
    //now that we are ticking, update should be written (and returned)
    let output_vals = mem.do_tick(); //this should have read_data and done, cuz write_en was hgih
    assert_eq!(output_vals.len(), 2);
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    let rd = output_vals.next().unwrap();
    let d = output_vals.next().unwrap();
    assert_eq!(rd.1.unwrap_imm().as_u64(), 16);
    assert_eq!(d.1.unwrap_imm().as_u64(), 1);
    //now try to overwrite but w/ write_en low, and see 16 and 0 is returned
    port_bindings![binds;
        write_data -> (3, 6),
        write_en -> (0, 1),
        addr0 -> (3, 2),
        addr1 -> (3, 2),
        addr2 -> (3, 2),
        addr3 -> (3, 2)
    ];
    let output_vals = mem.validate_and_execute(&binds);
    assert_eq!(1, output_vals.len()); //we should get read_data combinationally from [addr0]
    println!("output_vals: {:?}", output_vals);
    let mut output_vals = output_vals.into_iter();
    //should be a 16 and a 1 ([out] and [done])
    let out = output_vals.next().unwrap();
    let rd = out.1.unwrap_imm();
    assert_eq!(rd.as_u64(), 16);
}

/* #[test]
fn test_std_const() {
let val_31 = Value::try_from_init(31, 5).unwrap();
let const_31 = comb::StdConst::from_constants(5, val_31);
assert_eq!(const_31.read_val().as_u64(), 31); //can rust check this equality?
assert_eq!(const_31.read_u64(), 31);
}
#[test]
#[should_panic]
fn test_std_const_panic() {
let val = Value::try_from_init(75, 7).unwrap();
comb::StdConst::from_constants(5, val);
} */
#[test]
fn test_std_lsh() {
    // lsh with overflow
    // [11111] (31) -> [11100] (28)
    let mut lsh = comb::StdLsh::from_constants(5);
    port_bindings![binds;
        left -> (31, 5),
        right -> (2, 5)
    ];
    let out = lsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    println!("lsh of 31 by 2: {}", out);
    assert_eq!(out.as_u64(), 28);

    // lsh without overflow
    // lsh [010000] (16) by 1 -> [100000] (32)
    let mut lsh = comb::StdLsh::from_constants(6);
    port_bindings![binds;
        left -> (16, 6),
        right -> (1, 6)
    ];
    let out = lsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(out.as_u64(), 32);
}

#[test]
fn test_std_lsh_above64() {
    // lsh with overflow
    let mut lsh = comb::StdLsh::from_constants(275);
    port_bindings![binds;
        left -> (31, 275),
        right -> (275, 275)
    ];
    let out = lsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(out.as_u64(), 0);

    // lsh without overflow
    // lsh [010000] (16) by 1 -> [100000] (32)
    let mut lsh = comb::StdLsh::from_constants(381);
    port_bindings![binds;
        left -> (16, 381),
        right -> (1, 381)
    ];
    let out = lsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(out.as_u64(), 32);
}

#[test]
fn test_std_rsh() {
    // Not sure how to catagorize this
    // [1111] (15) -> [0011] (3)
    let mut rsh = comb::StdRsh::from_constants(4);
    port_bindings![binds;
        left -> (15, 4),
        right -> (2, 4)
    ];
    let out = rsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(out.as_u64(), 3);
    // Division by 2
    // [1000] (8) -> [0100] ( 4)
    port_bindings![binds;
        left -> (8, 4),
        right -> (1, 4)
    ];
    let out = rsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(out.as_u64(), 4);
}

#[test]
fn test_std_rsh_above64() {
    let mut rsh = comb::StdRsh::from_constants(275);
    port_bindings![binds;
        left -> (8, 275),
        right -> (4, 275)
    ];
    let out = rsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(out.as_u64(), 0);
    let mut rsh = comb::StdRsh::from_constants(381);
    port_bindings![binds;
        left -> (40, 381),
        right -> (3, 381)
    ];
    let out = rsh
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(out.as_u64(), 5);
}

#[test]
fn test_std_add() {
    // without overflow
    // add [0011] (3) and [1010] (10) -> [1101] (13)
    let mut add = comb::StdAdd::from_constants(4);
    port_bindings![binds;
        left -> (3, 4),
        right -> (10, 4)
    ];
    let res_add = add
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_add.as_u64(), 13);
    // with overflow
    // add [1010] (10) and [0110] (6) -> [0000] (0)
    port_bindings![binds;
        left -> (10, 4),
        right -> (6, 4)
    ];
    let res_add = add
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_add.as_u64(), 0);
}

#[test]
fn test_std_add_above64() {
    // without overflow
    let mut add = comb::StdAdd::from_constants(165);
    port_bindings![binds;
        left -> (17, 165),
        right -> (35, 165)
    ];
    let res_add = add
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_add.as_u64(), 52);
}

#[test]
#[should_panic]
fn test_std_add_panic() {
    let mut add = comb::StdAdd::from_constants(7);
    port_bindings![binds;
        left -> (81, 7),
        right -> (10, 4)
    ];
    add.validate_and_execute(&binds);
}
#[test]
fn test_std_sub() {
    // without overflow
    // sub [0110] (6) from [1010] (10) -> [0100] (4)
    let mut sub = comb::StdSub::from_constants(4);
    port_bindings![binds;
        left -> (10, 4),
        right -> (6, 4)
    ];
    let res_sub = sub
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_sub.as_u64(), 4);
    // with overflow (would produce a negative #, depending on how program thinks abt this...)
    // sub [1011] (11) from [1010] (10) ->  [1010] + [0101] = [1111] which is -1 in 2bc and 15 unsigned
    // for some reason producing [0101] ? that's just 'right + 1
    port_bindings![binds;
        left -> (10, 4),
        right -> (11, 4)
    ];
    let res_sub = sub
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_sub.as_u64(), 15);
    // sub [1111] (15) from [1000] (8) -> [1000] + [0001] which is [1001] -7 in 2c but 9 in unsigned

    port_bindings![binds;
        left -> (8, 4),
        right -> (15, 4)
    ];
    let res_sub = sub
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_sub.as_u64(), 9);
}

#[test]
fn test_std_sub_above64() {
    // without overflow
    let mut sub = comb::StdSub::from_constants(1605);
    port_bindings![binds;
        left -> (57, 1605),
        right -> (35, 1605)
    ];
    let res_sub = sub
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_sub.as_u64(), 22);
}

#[test]
#[should_panic]
fn test_std_sub_panic() {
    let mut sub = comb::StdAdd::from_constants(5);
    port_bindings![binds;
        left -> (52, 6),
        right -> (16, 5)
    ];
    sub.validate_and_execute(&binds);
}
#[test]
fn test_std_slice() {
    // 101 in binary is [1100101], take first 4 bits -> [0101] = 5
    let to_slice = Value::from(101, 7).unwrap();
    let mut std_slice = comb::StdSlice::from_constants(7, 4);
    let res_slice = std_slice
        .validate_and_execute(&[("in".into(), &to_slice)])
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm(); //note that once we implement execute_unary, have to change this
    assert_eq!(res_slice.as_u64(), 5);
    // Slice the entire bit
    let to_slice = Value::from(548, 10).unwrap();
    let mut std_slice = comb::StdSlice::from_constants(10, 10);
    let res_slice = std_slice
        .validate_and_execute(&[("in".into(), &to_slice)])
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_slice.as_u64(), 548);
}
#[test]
#[should_panic]
fn test_std_slice_panic() {
    let to_slice = Value::from(3, 2).unwrap();
    let mut std_slice = comb::StdSlice::from_constants(7, 4);
    std_slice.validate_and_execute(&[("in".into(), &to_slice)]);
}
#[test]
fn test_std_pad() {
    // Add 2 zeroes, should keep the same value
    let to_pad = Value::from(101, 7).unwrap();
    let mut std_pad = comb::StdPad::from_constants(7, 9);
    let res_pad = std_pad
        .validate_and_execute(&[("in".into(), &to_pad)])
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_pad.as_u64(), 101);
    // hard to think of another test case but just to have 2:
    let to_pad = Value::from(1, 7).unwrap();
    let res_pad = std_pad
        .validate_and_execute(&[("in".into(), &to_pad)])
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_pad.as_u64(), 1);
}
#[test]
#[should_panic]
fn test_std_pad_panic() {
    let to_pad = Value::from(21, 5).unwrap();
    let mut std_pad = comb::StdPad::from_constants(3, 9);
    std_pad.validate_and_execute(&[("in".into(), &to_pad)]);
}
/// Logical Operators
#[test]
fn test_std_not() {
    // ![1010] (!10) -> [0101] (5)
    let not0 = Value::from(10, 4).unwrap();
    let mut std_not = comb::StdNot::from_constants(4);
    let res_not = std_not
        .validate_and_execute(&[("in".into(), &not0)])
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_not.as_u64(), 5);
    // ![0000] (!0) -> [1111] (15)
    let not0 = Value::from(0, 4).unwrap();
    let res_not = std_not
        .validate_and_execute(&[("in".into(), &not0)])
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_not.as_u64(), 15);
}

#[test]
#[should_panic]
fn test_std_not_panic() {
    //input too short
    let not0 = Value::from(0, 4).unwrap();
    let mut std_not = comb::StdNot::from_constants(5);
    std_not
        .validate_and_execute(&[("in".into(), &not0)])
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
}

#[test]
fn test_std_and() {
    //101: [1100101], 78: [1001110] & -> [1000100] which is 68
    let mut std_and = comb::StdAnd::from_constants(7);
    port_bindings![binds;
        left -> (101, 7),
        right -> (78, 7)
    ];
    let res_and = std_and
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_and.as_u64(), 68);
    //[1010] (10) & [0101] (5) is [0000]

    let mut std_and = comb::StdAnd::from_constants(4);
    port_bindings![binds;
        left -> (10, 4),
        right -> (5, 4)
    ];
    let res_and = std_and
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_and.as_u64(), 0);
}

#[test]
#[should_panic]
fn test_std_and_panic() {
    let mut std_and = comb::StdAnd::from_constants(7);
    port_bindings![binds;
        left -> (91, 7),
        right -> (43, 6)
    ];
    std_and
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
}

#[test]
fn test_std_or() {
    //[101] (5) or [011] (3) is [111] (7)
    let mut std_or = comb::StdOr::from_constants(3);
    port_bindings![binds;
        left -> (5, 3),
        right -> (3, 3)
    ];
    let res_or = std_or
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_or.as_u64(), 7);
    //anything or zero is itself
    //[001] (1) or [000] (0) is [001] (1)
    port_bindings![binds;
        left -> (1, 3),
        right -> (0, 3)
    ];
    let res_or = std_or
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_or.as_u64(), left.as_u64());
}

#[test]
#[should_panic]
fn test_std_or_panic() {
    let mut std_or = comb::StdOr::from_constants(5);
    port_bindings![binds;
        left -> (16, 5),
        right -> (78, 7)
    ];
    std_or
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
}
#[test]
fn test_std_xor() {
    //[101] (5) XOR [011] (3) is [110] (6)
    let mut std_xor = comb::StdXor::from_constants(3);
    port_bindings![binds;
        left -> (5, 3),
        right -> (3, 3)
    ];
    let res_xor = std_xor
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_xor.as_u64(), 6);
    //anything xor itself is 0
    port_bindings![binds;
        left -> (5, 3),
        right -> (5, 3)
    ];
    assert_eq!(
        std_xor
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        0
    );
}
#[test]
#[should_panic]
fn test_std_xor_panic() {
    let mut std_xor = comb::StdXor::from_constants(6);
    port_bindings![binds;
        left -> (56, 6),
        right -> (92, 7)
    ];
    std_xor
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
}
/// Comparison Operators
// is there any point in testing this more than once?
// no weird overflow or anything. maybe test along with
// equals
#[test]
fn test_std_gt() {
    let mut std_gt = comb::StdGt::from_constants(16);
    port_bindings![binds;
        left -> (7 ,16),
        right -> (3, 16)
    ];
    let res_gt = std_gt
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_gt.as_u64(), 1);
    //7 > 7 ? no!
    port_bindings![binds;
        left -> (7, 16),
        right -> (7, 16)
    ];
    assert_eq!(
        std_gt
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        0
    );
}

#[test]
fn test_std_gt_above64() {
    let mut std_gt = comb::StdGt::from_constants(716);
    port_bindings![binds;
        left -> (18446744073709551615_u64, 716),
        right -> (14333, 716)
    ];
    let res_gt = std_gt
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_gt.as_u64(), 1);
    //7 > 7 ? no!
    let mut std_gt = comb::StdGt::from_constants(423);
    port_bindings![binds;
        left -> (7, 423),
        right -> (7, 423)
    ];
    assert_eq!(
        std_gt
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        0
    );
}
#[test]
#[should_panic]
fn test_std_gt_panic() {
    let mut std_gt = comb::StdGt::from_constants(3);
    port_bindings![binds;
        left -> (9, 4),
        right -> (3, 2)
    ];
    std_gt.validate_and_execute(&binds);
}
#[test]
fn test_std_lt() {
    let mut std_lt = comb::StdLt::from_constants(16);
    port_bindings![binds;
        left -> (7, 16),
        right -> (3, 16)
    ];
    let res_lt = std_lt
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_lt.as_u64(), 0);
    // 7 < 7 ? no!
    port_bindings![binds;
        left -> (7, 16),
        right -> (7, 16)
    ];
    assert_eq!(
        std_lt
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        0
    );
}

#[test]
fn test_std_lt_above64() {
    //7298791842 < 17298791842
    let mut std_lt = comb::StdLt::from_constants(2706);
    port_bindings![binds;
        left -> (72987918, 2706),
        right -> (18446744073709551615_u64, 2706)
    ];
    let res_lt = std_lt
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_lt.as_u64(), 1);
    //3_000_000 < 3_000_000 ? no!
    let mut std_lt = comb::StdLt::from_constants(2423);
    port_bindings![binds;
        left -> (3_000_000, 2423),
        right -> (3_000_000, 2423)
    ];
    assert_eq!(
        std_lt
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        0
    );
}

#[test]
#[should_panic]
fn test_std_lt_panic() {
    let mut std_lt = comb::StdLt::from_constants(5);
    port_bindings![binds;
        left -> (58, 6),
        right -> (12, 4)
    ];
    std_lt.validate_and_execute(&binds);
}
#[test]
fn test_std_eq() {
    let mut std_eq = comb::StdEq::from_constants(16);
    port_bindings![binds;
        left -> (4, 16),
        right -> (4, 16)
    ];
    let res_eq = std_eq
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_eq.as_u64(), 1);
    // 4 = 5 ? no!
    port_bindings![binds;
        left -> (4, 16),
        right -> (5, 16)
    ];
    assert_eq!(
        std_eq
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        0
    );
}

#[test]
fn test_std_eq_above64() {
    let mut std_eq = comb::StdEq::from_constants(716);
    port_bindings![binds;
        left -> (18446744073709551615_u64, 716),
        right -> (18446744073709551615_u64, 716)
    ];
    let res_eq = std_eq
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_eq.as_u64(), 1);
    // 123456 =12377456 ? no!
    let mut std_eq = comb::StdEq::from_constants(421113);
    port_bindings![binds;
        left -> (123456, 421113),
        right -> (12377456, 421113)
    ];
    assert_eq!(
        std_eq
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        0
    );
}

#[test]
#[should_panic]
fn test_std_eq_panic() {
    let mut std_eq = comb::StdEq::from_constants(5);
    port_bindings![binds;
        left -> (42, 6),
        right -> (42, 6)
    ];
    std_eq.validate_and_execute(&binds);
}
#[test]
fn test_std_neq() {
    let mut std_neq = comb::StdNeq::from_constants(16);
    port_bindings![binds;
        left -> (4, 16),
        right -> (4, 16)
    ];
    let res_neq = std_neq
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    //4 != 4 ? no!
    assert!(res_neq.as_u64() == 0);
    // 4 != 5? yes!
    port_bindings![binds;
        left -> (4, 16),
        right -> (5, 16)
    ];
    assert_eq!(
        std_neq
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        1
    );
}

#[test]
fn test_std_neq_above64() {
    let mut std_neq = comb::StdNeq::from_constants(4321);
    port_bindings![binds;
        left -> (18446744073709551615_u64, 4321),
        right -> (18446744073709551615_u64, 4321)
    ];
    let res_neq = std_neq
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    //max != max ? no!
    assert!(res_neq.as_u64() == 0);
    port_bindings![binds;
    left -> (18446744073709551615_u64, 4321),
    right -> (18446744073709500000_u64, 4321)
    ];
    assert_eq!(
        std_neq
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        1
    );
}

#[test]
#[should_panic]
fn test_std_neq_panic() {
    let mut std_neq = comb::StdNeq::from_constants(5);
    port_bindings![binds;
        left -> (45, 6),
        right -> (4, 3)
    ];
    std_neq.validate_and_execute(&binds);
}

#[test]
fn test_std_ge() {
    let mut std_ge = comb::StdGe::from_constants(8);
    port_bindings![binds;
        left -> (35, 8),
        right -> (165, 8)
    ];
    let res_ge = std_ge
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    //35 >= 165 ? no!
    assert_eq!(res_ge.as_u64(), 0);
    // 35 >= 35 ? yes
    port_bindings![binds;
        left -> (35, 8),
        right -> (35, 8)
    ];
    assert_eq!(
        std_ge
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        1
    );
}

#[test]
fn test_std_ge_above64() {
    let mut std_ge = comb::StdGe::from_constants(716);
    port_bindings![binds;
        left -> (18446744073709551615_u64, 716),
        right -> (14333, 716)
    ];
    let res_ge = std_ge
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_ge.as_u64(), 1);
    // 35 >= 35 ? yes
    let mut std_ge = comb::StdGe::from_constants(423);
    port_bindings![binds;
        left -> (35, 423),
        right -> (35, 423)
    ];
    assert_eq!(
        std_ge
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        1
    );
}

#[test]
#[should_panic]
fn test_std_ge_panic() {
    let mut std_ge = comb::StdGe::from_constants(6);
    port_bindings![binds;
        left -> (40, 6),
        right -> (75, 7)
    ];
    std_ge.validate_and_execute(&binds);
}
#[test]
fn test_std_le() {
    let mut std_le = comb::StdLe::from_constants(4);
    port_bindings![binds;
        left -> (12, 4),
        right -> (8, 4)
    ];
    let res_le = std_le
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    //12 <= 4 ? no!
    assert_eq!(res_le.as_u64(), 0);
    //12 <= 12? yes!
    port_bindings![binds;
        left -> (12, 4),
        right -> (12, 4)
    ];
    assert_eq!(
        std_le
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        1
    );
}

#[test]
fn test_std_le_above64() {
    //72987918 <= 9729879
    let mut std_le = comb::StdLe::from_constants(2706);
    port_bindings![binds;
        left -> (72_987_918, 2706),
        right -> (93_729_879, 2706)
    ];
    let res_le = std_le
        .validate_and_execute(&binds)
        .into_iter()
        .next()
        .map(|(_, v)| v)
        .unwrap()
        .unwrap_imm();
    assert_eq!(res_le.as_u64(), 1);
    //3_000_000 <= 3_000_000 ? yes!
    let mut std_le = comb::StdLe::from_constants(2423);
    port_bindings![binds;
        left -> (3_000_000, 2423),
        right -> (3_000_000, 2423)
    ];
    assert_eq!(
        std_le
            .validate_and_execute(&binds)
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .unwrap()
            .unwrap_imm()
            .as_u64(),
        1
    );
}

#[test]
#[should_panic]
fn test_std_le_panic() {
    let mut std_le = comb::StdLe::from_constants(6);
    port_bindings![binds;
        left -> (93, 7),
        right -> (68, 7)
    ];
    std_le.validate_and_execute(&binds);
}
