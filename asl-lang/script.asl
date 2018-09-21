state("bgb.exe") {
    y: u8 = "bgb.exe", 0x00166EDC, 0x274, 0x361;
    x: u8 = "bgb.exe", 0x00166EDC, 0x274, 0x362;
}

start {
    let a: u16 = 2345;
    let b: u32 = 567;
    let eq = a as u8 < b as _;

    let c = a as u8;
    let d = c + c;
    let e = d - d;

    let okokok: i64 = 257 << 3;

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

    let isOnSpot = current.x == 7 && current.y == 1;
    let wasOnSpot = old.x == 7 && old.y == 1;
    isOnSpot && !wasOnSpot
}

split {
    let isOnSpot = current.x == 5 && current.y == 1;
    let wasOnSpot = old.x == 5 && old.y == 1;
    isOnSpot && !wasOnSpot
}

reset {
    let isOnSpot = current.x == 3 && current.y == 2;
    let wasOnSpot = old.x == 3 && old.y == 2;
    isOnSpot && !wasOnSpot
}
