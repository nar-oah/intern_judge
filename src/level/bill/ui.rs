use super::DailyBill;
use crate::level::{SceneRoot, ui::confirm_input_scene};
use bevy::{prelude::*, text::FontSourceTemplate};

pub(super) fn bill_scene(bill: DailyBill) -> impl Scene {
    let DailyBill {
        balance,
        salary,
        rent,
        transactions,
        total,
    } = &bill;
    let summary = format!(
        "每日账单\n当前余额：{balance}\n本周工资：{salary}\n房租：{rent}\n便利店付款：{}\n工具店付款：{}\n救助猫咪收益：{}\n彩票店付款：{}\n彩票中奖收益：{}\n结算后余额：{total}\n按 E 确认",
        transactions.convenience_store_cost,
        transactions.tool_store_cost,
        transactions.cat_reward,
        transactions.lottery_cost,
        transactions.lottery_reward,
    );
    bsn! {
        #DailyBill
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
