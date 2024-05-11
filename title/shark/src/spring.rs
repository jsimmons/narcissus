use narcissus_maths::exp_f32;

// https://theorangeduck.com/page/spring-roll-call
pub fn simple_spring_damper_exact(
    x: f32,
    velocity: f32,
    goal: f32,
    damping: f32,
    delta_time: f32,
) -> (f32, f32) {
    let y = damping / 2.0;
    let j0 = x - goal;
    let j1 = velocity + j0 * y;
    let eydt = exp_f32(-y * delta_time);
    (
        eydt * (j0 + j1 * delta_time) + goal,
        eydt * (velocity - j1 * y * delta_time),
    )
}
