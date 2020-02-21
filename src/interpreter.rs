pub fn run(codes: &Vec<u8>, constants: &Vec<String>) {
    println!("hge\"");
    let mut ip = 0;
    loop {
        println!("{}", codes[ip]);
        ip += 1;
    }
    /*Code::PushString(s) => {
        let index = constants.insert_full(s.clone()).0;
        [vec![1], Code::usize_to_bytes(index)].concat()
    }
    Code::PushBoolean(b) => vec![2, *b as u8],
    Code::PushFloat(f) => [vec![3], Code::f64_to_bytes(*f)].concat(),
    Code::PushInt(i) => [vec![4], Code::u64_to_bytes(*i)].concat(),
    Code::Store(name) => {
        let index = constants.insert_full(name.clone()).0;
        [vec![6], Code::usize_to_bytes(index)].concat()
    }
    Code::Load(name) => {
        let index = constants.insert_full(name.clone()).0;
        [vec![7], Code::usize_to_bytes(index)].concat()
    }
    Code::TStore(i) => {
        let index = constants.insert_full(format!("<{}>", i)).0;
        [vec![6], Code::usize_to_bytes(index)].concat()
    }
    Code::TLoad(i) => {
        let index = constants.insert_full(format!("<{}>", i)).0;
        [vec![7], Code::usize_to_bytes(index)].concat()
    }
    Code::JumpNot(p) => [vec![10], Code::usize_to_bytes(*p)].concat(),
    Code::Goto(p) | Code::Break(p) => [vec![11], Code::usize_to_bytes(*p)].concat(),
    Code::NewFun(args, codes) => {
        let arglen = Code::usize_to_bytes(args.len());
        let argindices: Vec<u8> = args
            .iter()
            .map(|arg| Code::usize_to_bytes(constants.insert_full(arg.clone()).0))
            .flat_map(|bytes| bytes)
            .collect();
        let codeslen = Code::usize_to_bytes(codes.len());
        [vec![5], arglen, argindices, codeslen, codes.clone()].concat()
    }
    Code::PushNone => vec![12],
    Code::NewBendy => vec![13],
    Code::NewList => vec![14],
    Code::Return => vec![15],
    Code::Neg => vec![16],
    Code::Add => vec![17],
    Code::Sub => vec![18],
    Code::Mul => vec![19],
    Code::IntDiv => vec![20],
    Code::FloatDiv => vec![21],
    Code::Mod => vec![22],
    Code::BitLsh => vec![23],
    Code::BitRsh => vec![24],
    Code::BitURsh => vec![25],
    Code::BitAnd => vec![26],
    Code::BitOr => vec![27],
    Code::BitXOr => vec![28],
    Code::BoolNot => vec![29],
    Code::Concat => vec![30],
    Code::Put => vec![31],
    Code::Get => vec![32],
    Code::Call => vec![33],
    Code::BoolAnd => vec![34],
    Code::BoolOr => vec![35],
    Code::Equals => vec![36],
    Code::NotEquals => vec![37],
    Code::LessThan => vec![38],
    Code::LessEquals => vec![39],
    Code::GreaterThan => vec![40],
    Code::GreaterEquals => vec![41],*/
}
