//! 手牌系统——玩家干预世界的有限机会

/// 手牌类型
#[derive(Debug, Clone)]
pub enum HandCard {
    /// 砸——对目标施加一次力（Strike）或一次加工（Alter）
    /// 前提：手上拿着正确工具（或空手）+ 目标实体
    Strike,

    /// 拿——抓起一张卡到空中，选择放置位置
    PickUp,

    /// 抽——从叠加的多单元卡中分离一张（Separate）
    Separate,

    /// 叠——把手中卡放到格子或另一张卡上（Combine）
    Combine,

    /// 跳过时间——世界快进 N 天
    TimeSkip { days: u32 },
}

/// 玩家手牌槽
#[derive(Debug, Clone)]
pub struct PlayerHand {
    pub cards: Vec<HandCard>,
    pub max_size: usize,
}

impl PlayerHand {
    pub fn new(max_size: usize) -> Self {
        Self { cards: Vec::new(), max_size }
    }

    /// 添加手牌（满了返回 false）
    pub fn add(&mut self, card: HandCard) -> bool {
        if self.cards.len() >= self.max_size { return false; }
        self.cards.push(card);
        true
    }

    /// 使用第 index 张手牌（消耗）
    pub fn use_card(&mut self, index: usize) -> Option<HandCard> {
        if index < self.cards.len() {
            Some(self.cards.remove(index))
        } else {
            None
        }
    }

    pub fn is_full(&self) -> bool { self.cards.len() >= self.max_size }
    pub fn is_empty(&self) -> bool { self.cards.is_empty() }
    pub fn count(&self) -> usize { self.cards.len() }
}

/// 手牌操作执行结果
#[derive(Debug)]
pub enum HandCardResult {
    /// 操作成功
    Success,
    /// 被公理阻止
    Blocked { reason: String },
    /// 无效操作
    Invalid,
    /// 时间跳过完成
    TimeSkipComplete { days_skipped: u32, events_count: usize },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_hand_add_and_count() {
        let mut hand = PlayerHand::new(3);
        assert!(hand.is_empty());
        assert_eq!(hand.count(), 0);

        assert!(hand.add(HandCard::Strike));
        assert_eq!(hand.count(), 1);
        assert!(!hand.is_empty());
        assert!(!hand.is_full());

        assert!(hand.add(HandCard::PickUp));
        assert!(hand.add(HandCard::Combine));
        assert_eq!(hand.count(), 3);
        assert!(hand.is_full());

        // Full — cannot add more
        assert!(!hand.add(HandCard::Separate));
        assert_eq!(hand.count(), 3);
    }

    #[test]
    fn player_hand_use_card() {
        let mut hand = PlayerHand::new(5);
        hand.add(HandCard::Strike);
        hand.add(HandCard::PickUp);
        hand.add(HandCard::Separate);

        // Use the second card (index 1 = PickUp)
        let card = hand.use_card(1);
        assert!(card.is_some());
        assert!(matches!(card.unwrap(), HandCard::PickUp));
        assert_eq!(hand.count(), 2);

        // Out-of-bounds returns None
        assert!(hand.use_card(10).is_none());
        assert_eq!(hand.count(), 2);
    }

    #[test]
    fn player_hand_is_full() {
        let mut hand = PlayerHand::new(2);
        assert!(!hand.is_full());
        hand.add(HandCard::Strike);
        assert!(!hand.is_full());
        hand.add(HandCard::Combine);
        assert!(hand.is_full());
    }

    #[test]
    fn player_hand_time_skip_card() {
        let mut hand = PlayerHand::new(5);
        hand.add(HandCard::TimeSkip { days: 3 });
        assert_eq!(hand.count(), 1);
        let card = hand.use_card(0).unwrap();
        assert!(matches!(card, HandCard::TimeSkip { days: 3 }));
        assert!(hand.is_empty());
    }
}
