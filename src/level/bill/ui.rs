use super::WeeklyBill;
use crate::level::{SceneRoot, ui::confirm_input_scene};
use bevy::{prelude::*, text::FontSourceTemplate};

pub(super) fn bill_scene(balance: i32, salary: i32, rent: i32, bill: WeeklyBill) -> impl Scene {
    let summary = format!(
        "周五账单\n当前余额：{balance}\n本周工资：{salary}\n房租：{rent}\n结算后余额：{}\n按 E 确认",
        bill.total,
    );
    bsn! {
        #WeeklyBill
        SceneRoot
        bill
        @confirm_input_scene()
        Node {
            width: percent(100),
            height: percent(100),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
        }
        GlobalZIndex(300)
        BackgroundColor(Color::srgb(0.04, 0.05, 0.08))
        Pickable::default()
        Children [
            Text::new(summary)
            TextFont {
                font: FontSourceTemplate::Handle("fonts/court-dialogue.ttf"),
                font_size: FontSize::Px(28.0),
            }
            TextColor(Color::srgb(0.95, 0.96, 1.0))
            Pickable::IGNORE
        ]
    }
}
