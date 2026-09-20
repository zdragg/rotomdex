pub mod contest_type {
    crate::endpoint!(crate::model::contests::ContestType; for "contest-type");
}

pub mod contest_effect {
    crate::endpoint!(unnamed crate::model::contests::ContestEffect; for "contest-effect");
}

pub mod super_contest_effect {
    crate::endpoint!(unnamed crate::model::contests::SuperContestEffect; for "super-contest-effect");
}
