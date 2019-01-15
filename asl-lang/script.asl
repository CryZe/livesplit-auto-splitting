state("bgb.exe") {
    y: u8 = "bgb.exe", 0x00166EDC, 0x274, 0x361;
    x: u8 = "bgb.exe", 0x00166EDC, 0x274, 0x362;
}

start {
    let a = 2345;
    let b: u32 = 567;
    let eq = a < b as _;

    let c = a as u8;
    let d = c + c;
    let e = d - d;

    let okokok: i64 = 257 << 3;

    let lul = 234;
    let lmao: (i16, _, u8, _, f64) = (lul, 2 & lul, 3, true, 3 + 1);

    {} == {};

    let okok = match okokok {
        2 => false,
        1 | 4..7 | 10..=20 => true,
        3 => true,
        _ => {
            let y = 3;
            y == 4
        }
    };

    enteredSpot(7, 1)
}

split {
    enteredSpot(5, 1)
}

reset {
    enteredSpot(3, 2)
}

fn enteredSpot(x, y) {
    let isOnSpot = current.x == x && current.y == y;
    let wasOnSpot = old.x == x && old.y == y;
    isOnSpot && !wasOnSpot
}
