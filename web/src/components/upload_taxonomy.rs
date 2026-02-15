pub fn tid_options() -> &'static [(u16, &'static str)] {
    &[
        (171, "电子竞技"),
        (17, "单机游戏"),
        (27, "综合动画"),
        (24, "音乐"),
        (65, "生活"),
        (95, "科技"),
        (182, "影视"),
    ]
}

pub fn tid_name(tid: u16) -> String {
    tid_options()
        .iter()
        .find(|(id, _)| *id == tid)
        .map(|(_, name)| (*name).to_string())
        .unwrap_or_else(|| tid.to_string())
}
