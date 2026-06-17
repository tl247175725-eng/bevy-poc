//! 卡牌棋子底座
//!
//! 统一黑圆柱底座 + 数字标签。所有卡牌棋子共用。
//! - 底座：黑色扁平圆柱（半径 0.3，高 0.05）
//! - 数字标签：白字，只在 quantity > 1 时显示

use bevy::math::primitives::Cylinder;
use bevy::prelude::*;

/// 底座上的数量标签（白字）
#[derive(Component)]
pub struct BaseQuantityLabel;

/// 生成底座 mesh：黑色扁平圆柱。
/// 所有卡共用同一个 mesh——GPU 自动实例化。
pub fn generate_base_mesh() -> Mesh {
    Cylinder::new(0.3, 0.025).into()  // radius, half_height
}

/// 生成纯黑材质。
pub fn generate_base_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::BLACK,
        ..default()
    }
}

/// 生成底座标准材质 handle。
pub fn create_base_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    materials.add(generate_base_material())
}

/// 生成底座 mesh handle。
pub fn create_base_mesh(meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
    meshes.add(generate_base_mesh())
}

/// 在一个位置生成底座圆柱 + 可选数量标签。
///
/// - `pos`: 底座中心位置（地表 Y 坐标）
/// - `quantity`: 卡牌叠放数量。`<= 1` 时不显示数字
/// - `base_mesh`: 底座 mesh handle
/// - `base_material`: 底座材质 handle
/// - `font`: 字体 handle（`Handle::default()` 使用内置字体）
pub fn spawn_base_with_label(
    commands: &mut Commands,
    pos: Vec3,
    quantity: u32,
    base_mesh: Handle<Mesh>,
    base_material: Handle<StandardMaterial>,
    font: Handle<Font>,
) {
    // 底座圆柱
    commands.spawn((
        Mesh3d(base_mesh),
        MeshMaterial3d(base_material),
        Transform::from_translation(pos),
    ));

    // 数字标签——只在 quantity > 1 时显示
    if quantity > 1 {
        let label_pos = pos + Vec3::Y * 0.025; // 贴在底座上表面
        commands.spawn((
            Text2d::new(quantity.to_string()),
            TextFont {
                font,
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_translation(label_pos).with_scale(Vec3::splat(0.005)),
            BaseQuantityLabel,
        ));
    }
}
