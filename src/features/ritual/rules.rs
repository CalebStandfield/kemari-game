pub fn touch_base_reward(kind: crate::core::TouchKind) -> f32 {
    match kind {
        crate::core::TouchKind::Kick => 1.8,
        crate::core::TouchKind::Head => 4.0,
        crate::core::TouchKind::Juggle => 3.0,
    }
}
