use debug_info::{Hover, Span};
use types::Ty;
use {compile, hover};

#[test]
fn test_hover() {
    let result = hover(
        r#"
state("game") {}

start {
    let xyz = 5;

    let y: u8 = 7 +
    xyz;

    true
}
"#,
        7,
        19,
    ).unwrap()
    .unwrap();

    let expected = Hover {
        entity: result.entity,
        params: None,
        ty: Ty::U8,
        span: Span {
            from: (7, 17),
            to: (8, 8),
        },
    };

    assert_eq!(result, expected);
}

#[test]
fn normal_compile() {
    compile(
        r#"state("game.exe") {
}

start {
    false
}

split {
    true
}"#,
    ).unwrap();
}

#[test]
fn number_literals() {
    compile(
        r#"state("game.exe") {
}

start {
    let x = 5;
    false
}

split {
    true
}"#,
    ).unwrap();
}

#[test]
fn function_call() {
    compile(
        r#"state("game.exe") {
}

start {
    false
}

split {
    let x: f64 = foo(5);
    true
}

fn foo(y) {
    let x = foo(4);
    4
}"#,
    ).unwrap();
}

#[test]
fn casts() {
    compile(
        r#"state("game.exe") {
}

split {
    let a: i32 = 5;
    let b = 5 as f64;
    let x = foo(a) as f64;
    true
}

fn foo(a) {
    a
}"#,
    ).unwrap();
}

#[test]
fn too_many_function_args() {
    compile(
        r#"state("game.exe") {
}

split {
    foo(1, 2);
    true
}

fn foo(a) {
    a
}"#,
    ).unwrap_err();
}

#[test]
fn too_few_function_args() {
    compile(
        r#"state("game.exe") {
}

split {
    foo(1, 2);
    true
}

fn foo(a, b, c: f64) {
    a
}"#,
    ).unwrap_err();

    compile(
        r#"state("game.exe") {
}

split {
    foo(1, 2);
    true
}

fn foo(a, b, c) {
    a
}"#,
    ).unwrap_err();
}
