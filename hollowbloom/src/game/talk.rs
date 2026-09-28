//! Talking with the townsfolk: conversations with little voices, gifts, presents between
//! friends, taking and handing in quests, the notice boards, the journal and the
//! celebration when a quest is done.

use glam::{Vec2, Vec3};

use super::Io;
use super::fish::{self, FISH};
use super::folk::{FOLK, Taste, VILLAGERS, Villager};
use super::gear::{Group, Rarity};
use super::items::{Item, Kind, Stack};
use super::menus::{Menu, center, inside, tabs};
use super::play::Play;
use super::quests::{Goal, QUESTS, QuestId, RANKS, Reward, goal_text, project_name, reward_text};
use super::town;
use super::world::Area;
use crate::assets::Assets;
use crate::audio::Sfx;
use crate::input::{Action, Button};
use crate::palette::*;
use crate::render::Camera;
use crate::ui::{Canvas, Style};
use crate::util::hash2;

/// A choice in a conversation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Say {
    Shop,
    /// Learn spells and choose the two you carry (Hazel only).
    Spells,
    Chat,
    Gift,
    /// Ask about a story quest (index into `QUESTS`).
    Offer(usize),
    Accept(usize),
    Later,
    /// How an open quest is going (index into your quests).
    About(usize),
    Bye,
}

/// What each villager gives you at three, six and nine hearts.
fn presents(v: Villager) -> [Reward; 3] {
    use Reward::{Gear, Item as It, Scroll};
    match v {
        Villager::Thistle => [
            It(Item::PumpkinPie, 2),
            It(Item::HeartCrystal, 1),
            It(Item::WishStar, 1),
        ],
        Villager::Rowan => [
            Gear(Item::IronShield, Rarity::Rare),
            Scroll(Group::Weapon, Rarity::Epic),
            It(Item::HeartCrystal, 1),
        ],
        Villager::Quill => [
            Scroll(Group::Armor, Rarity::Rare),
            Scroll(Group::Weapon, Rarity::Epic),
            Scroll(Group::Tool, Rarity::Legendary),
        ],
        Villager::Hilde => [
            Gear(Item::LeatherBoots, Rarity::Rare),
            Gear(Item::IronPlate, Rarity::Epic),
            Gear(Item::CrystalMail, Rarity::Legendary),
        ],
        Villager::Garrick => [
            Gear(Item::MightyLeek, Rarity::Rare),
            Gear(Item::FrostFang, Rarity::Epic),
            Gear(Item::BoneSabre, Rarity::Legendary),
        ],
        Villager::Nix => [
            It(Item::Sprinkler, 3),
            It(Item::QualitySprinkler, 4),
            It(Item::CrystalSprinkler, 4),
        ],
        Villager::Opal => [
            It(Item::Ruby, 2),
            It(Item::Moonstone, 2),
            It(Item::StarDiamond, 1),
        ],
        Villager::Wren => [
            It(Item::Lamp, 3),
            It(Item::Chest, 2),
            It(Item::EnchantTable, 1),
        ],
        Villager::Posy => [
            It(Item::RoseSeeds, 10),
            It(Item::MoonbloomSeeds, 10),
            It(Item::StarfruitSeeds, 5),
        ],
        Villager::Mabel => [
            It(Item::BlueberryMuffin, 5),
            It(Item::Shortcake, 5),
            It(Item::StarfruitTart, 3),
        ],
        Villager::Barley => [
            It(Item::VeggieStew, 5),
            It(Item::EmberCurry, 5),
            It(Item::TruffleRisotto, 3),
        ],
        Villager::Fern => [
            It(Item::PlumPudding, 2),
            Gear(Item::WoollyPoncho, Rarity::Epic),
            It(Item::HeartCrystal, 1),
        ],
        Villager::Pip => [
            It(Item::GlassMarble, 1),
            It(Item::RubberDuck, 1),
            It(Item::TinyCrown, 1),
        ],
        Villager::Juniper => [
            It(Item::GlowcapSpores, 10),
            It(Item::JellySpores, 10),
            It(Item::TruffleSpores, 5),
        ],
        Villager::Bramble => [
            It(Item::Popcorn, 5),
            It(Item::MusicBox, 1),
            It(Item::WishStar, 1),
        ],
        Villager::Toby => [
            It(Item::Feather, 3),
            It(Item::MintTea, 3),
            Gear(Item::FrogCan, Rarity::Epic),
        ],
        Villager::Clank => [
            Gear(Item::IronHelm, Rarity::Rare),
            Gear(Item::FrostCoat, Rarity::Epic),
            Gear(Item::FrostWard, Rarity::Legendary),
        ],
        Villager::Mira => [
            It(Item::WishStar, 1),
            It(Item::Moonstone, 2),
            It(Item::StarFossil, 1),
        ],
        Villager::Olive => [
            It(Item::Sprinkler, 2),
            It(Item::QualitySprinkler, 3),
            Gear(Item::GoldHoe, Rarity::Epic),
        ],
        Villager::Hazel => [
            It(Item::LargeManaPotion, 5),
            Reward::Spell(super::spells::Spell::Blink),
            Gear(Item::MoonpetalWand, Rarity::Legendary),
        ],
    }
}

/// What folk say about a charming home.
const HOUSE_TALK: [&str; 6] = [
    "I walked past your farmhouse last night. The windows were glowing. So cozy!",
    "Wren says your house is the most charming in the valley. High praise!",
    "Is it true you have fish in a tank? Can I visit them sometime?",
    "Your home always smells of cooking. Makes me hungry just thinking about it.",
    "You've got taste, farmer. Real taste.",
    "Pip won't stop talking about your house. Something about 'Captain'?",
];

/// Something small folk bring a charming neighbour.
fn charm_gift(v: Villager) -> Item {
    match v {
        Villager::Thistle => Item::FreshBread,
        Villager::Rowan => Item::Feather,
        Villager::Quill => Item::ManaTonic,
        Villager::Hilde => Item::HealingTonic,
        Villager::Garrick => Item::Bait,
        Villager::Nix => Item::Sprinkler,
        Villager::Opal => Item::Topaz,
        Villager::Wren => Item::PottedFern,
        Villager::Posy => Item::TomatoSeeds,
        Villager::Mabel => Item::BlueberryMuffin,
        Villager::Barley => Item::FishAndChips,
        Villager::Fern => Item::PlumPudding,
        Villager::Pip => Item::GlassMarble,
        Villager::Juniper => Item::GlowcapSpores,
        Villager::Bramble => Item::Popcorn,
        Villager::Toby => Item::Feather,
        Villager::Clank => Item::HealingTonic,
        Villager::Mira => Item::MintTea,
        Villager::Olive => Item::Rose,
        Villager::Hazel => Item::SmallManaPotion,
    }
}

const PRESENT_LINES: [&str; 3] = [
    "Oh, before I forget - I saw this and thought of you. Take it!",
    "You mean a lot to me, you know. I want you to have this.",
    "You're one of my dearest friends in the whole world. Please, take this.",
];

/// Something that can be given as a gift.
fn giftable(s: &Stack) -> bool {
    !matches!(
        s.item.def().kind,
        Kind::Keepsake | Kind::Coin(_) | Kind::Gear(_) | Kind::Scroll(_)
    )
}

impl Play {
    /// A heart milestone present they haven't given you yet (0, 1 or 2).
    pub fn present_due(&self, v: Villager) -> Option<usize> {
        let h = self.friends.hearts(v);
        let got = self.friends.presents[v as usize];
        (0..3).find(|&k| h >= [3, 6, 9][k] && got & (1 << k) == 0)
    }

    /// Starts a conversation with someone nearby.
    pub fn talk_to(&mut self, i: usize, io: &mut Io) {
        let who = self.folk[i].who;
        for n in &mut self.folk {
            n.talking = false;
        }
        self.folk[i].talking = true;
        let d = who.def();
        let vi = who as usize;
        let mut text;
        if !self.friends.met[vi] {
            self.friends.met[vi] = true;
            self.friends.add(who, 10);
            text = d.hello.to_string();
            // Hazel starts every delver off with a spell.
            if who == Villager::Hazel && self.spells.learn(super::spells::Spell::Firebolt) {
                io.audio.play(Sfx::SpellUp);
                self.toast_colored("Learned Firebolt! Press Q to cast it.", None, 0, GOLD);
            }
        } else if let Some(qi) = self.quest_to_finish(who) {
            text = self.quests[qi]
                .story()
                .map_or("Thank you so much!".to_string(), |q| q.thanks.to_string());
            self.finish_quest(qi, io);
        } else if let Some(k) = self.present_due(who) {
            self.friends.presents[vi] |= 1 << k;
            text = PRESENT_LINES[k].to_string();
            let r = presents(who)[k];
            if let Some(line) = self.grant(&r, io) {
                self.cheer = Some(super::quests::Cheer {
                    title: format!("A gift from {}", who.name()),
                    who,
                    lines: vec![line],
                    t: 0.0,
                });
            }
            io.audio.play(Sfx::Heart);
        } else {
            text = self.chat_line(who);
        }
        if !self.friends.talked[vi] {
            self.friends.talked[vi] = true;
            // Folk who've heard about your lovely home sometimes bring a little something.
            let charm = self.charisma();
            if charm >= 20 && self.rng.chance(charm.min(80) as f32 / 320.0) {
                let gift = charm_gift(who);
                self.give(Stack::new(gift, 1));
                self.toast_colored(
                    format!("{} gave you {}!", who.name(), gift.def().name),
                    Some(gift),
                    0,
                    PINK,
                );
                text = format!(
                    "Everyone's talking about your lovely home! I brought you a little \
                     something. {text}"
                );
            }
            let n = self.charm_friend(20);
            let up = self.friends.add(who, n);
            let at = self.folk[i].world_pos() + Vec3::Y * 1.2;
            self.fx.popup(at, "♥", PINK);
            if up {
                io.audio.play(Sfx::Heart);
                self.toast_colored(
                    format!("{} ♥ {}", who.name(), self.friends.hearts(who)),
                    None,
                    0,
                    PINK,
                );
            }
        }
        self.on_meet(who);
        if text.is_empty() {
            text = "Hello!".into();
        }
        let choices = self.talk_choices(who);
        self.menu = Menu::Talk {
            who,
            text,
            shown: 0.0,
            choices,
            sel: 0,
            blip: 0.0,
        };
    }

    fn chat_line(&self, who: Villager) -> String {
        let d = who.def();
        let k = hash2(self.clock.day as i32, (self.time * 3.0) as i32, who as u32) as usize;
        // A charming home gets talked about.
        let charm = self.charisma();
        if charm >= 30 && k % 5 == 1 {
            return HOUSE_TALK[(k / 5) % HOUSE_TALK.len()].to_string();
        }
        if self.friends.hearts(who) >= 6 && k % 3 == 0 {
            d.close[k % d.close.len()].to_string()
        } else {
            d.chat[k % d.chat.len()].to_string()
        }
    }

    fn talk_choices(&self, who: Villager) -> Vec<(String, Say)> {
        let mut v = Vec::new();
        if let (Some(p), Area::Inside(here)) = (who.def().keeps, self.area) {
            if p == here && p != town::Place::Hall {
                if p == town::Place::Spellery {
                    v.push(("Spells".to_string(), Say::Spells));
                }
                v.push(("Let's trade".to_string(), Say::Shop));
            }
        }
        if let Some(d) = self.quest_for(who) {
            let i = QUESTS.iter().position(|q| q.key == d.key).unwrap_or(0);
            let _ = d;
            v.push(("! Can I help?".to_string(), Say::Offer(i)));
        }
        if let Some(i) = self
            .quests
            .iter()
            .position(|q| q.request().is_none() && q.giver() == who)
        {
            v.push(("About your request...".to_string(), Say::About(i)));
        }
        let vi = who as usize;
        if !self.friends.gifted[vi] {
            if let Some(s) = self.player.held_stack().filter(|s| giftable(s)) {
                v.push((format!("Give {}", s.item.def().name), Say::Gift));
            }
        }
        v.push(("Chat".to_string(), Say::Chat));
        v.push(("Bye!".to_string(), Say::Bye));
        v
    }

    /// Acts on a conversation choice. Returns the next line to say, or `None` to end.
    pub fn talk_choice(
        &mut self,
        who: Villager,
        say: Say,
        io: &mut Io,
    ) -> Option<(String, Vec<(String, Say)>)> {
        match say {
            Say::Shop => {
                if let Some(p) = who.def().keeps {
                    self.menu = Menu::shop_at(p);
                }
                None
            }
            Say::Spells => {
                self.menu = Menu::Spells { tab: 0, sel: 0 };
                None
            }
            Say::Chat => Some((self.chat_line(who), self.talk_choices(who))),
            Say::Gift => {
                let Some(s) = self.player.held_stack().filter(|s| giftable(s)).copied() else {
                    return Some(("Hm?".into(), self.talk_choices(who)));
                };
                let sel = self.player.sel;
                self.player.inv.take_one(sel);
                self.friends.gifted[who as usize] = true;
                let taste = who.taste(s.item);
                let n = self.charm_friend(taste.points());
                let up = self.friends.add(who, n);
                match taste {
                    Taste::Love | Taste::Like => io.audio.play(Sfx::Heart),
                    Taste::Hate => io.audio.play(Sfx::Denied),
                    Taste::Fine => io.audio.play(Sfx::Pickup),
                }
                if let Some(n) = self.folk.iter().find(|n| n.who == who) {
                    let at = n.world_pos() + Vec3::Y * 1.1;
                    let (txt, col) = match taste {
                        Taste::Love => ("♥♥♥", PINK),
                        Taste::Like => ("♥♥", PINK),
                        Taste::Fine => ("♥", BLUSH),
                        Taste::Hate => ("...", SHADOW),
                    };
                    self.fx.popup(at, txt, col);
                    if taste == Taste::Love {
                        self.fx.motes(at, 12, &[PINK, BLUSH, WHITE], 0.4);
                    }
                }
                if up {
                    self.toast_colored(
                        format!("{} ♥ {}", who.name(), self.friends.hearts(who)),
                        None,
                        0,
                        PINK,
                    );
                }
                Some((taste.reply(who, s.item.def().name), self.talk_choices(who)))
            }
            Say::Offer(i) => {
                let d = &QUESTS[i];
                let mut t = d.ask.to_string();
                t.push_str("\n> ");
                t.push_str(&goal_text(d.goal));
                Some((
                    t,
                    vec![
                        ("I'll do it!".to_string(), Say::Accept(i)),
                        ("Maybe later".to_string(), Say::Later),
                    ],
                ))
            }
            Say::Accept(i) => {
                let d = &QUESTS[i];
                let ok = self.accept(QuestId::Story(d.key.to_string()), io);
                let t = if ok {
                    "Thank you! Come and find me when it's done - I'll be ever so grateful."
                } else {
                    "Oh, your bag's full! Make some room and ask me again."
                };
                Some((t.to_string(), self.talk_choices(who)))
            }
            Say::Later => Some((
                "No rush at all. The offer stands!".into(),
                self.talk_choices(who),
            )),
            Say::About(i) => {
                let q = self.quests.get(i)?;
                let (have, need) = self.progress(q);
                let t = format!(
                    "How's \"{}\" coming along? {} ({}/{}). I'll be right here!",
                    q.title(),
                    goal_text(q.goal()),
                    have,
                    need
                );
                Some((t, self.talk_choices(who)))
            }
            Say::Bye => None,
        }
    }

    /// Runs the conversation menu.
    #[allow(clippy::too_many_arguments)]
    pub fn update_talk(
        &mut self,
        io: &mut Io,
        who: Villager,
        text: String,
        mut shown: f32,
        choices: Vec<(String, Say)>,
        mut sel: usize,
        mut blip: f32,
    ) -> Menu {
        let input = io.input;
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        let lclick = input.button_pressed(Button::Left);
        let total = text.chars().count() as f32;
        let skip = input.pressed(Action::Confirm) || lclick;
        if shown < total {
            let before = shown as usize;
            shown += io.dt * 55.0;
            if skip {
                shown = total;
            }
            // A little voice, one blip every few letters.
            blip += (shown as usize - before) as f32;
            if blip >= 3.0 && shown < total {
                blip = 0.0;
                let wob = (hash2(shown as i32, who as i32, 3) % 100) as f32 / 100.0;
                io.audio
                    .play_at(Sfx::Talk, 0.35, who.def().voice * (0.92 + wob * 0.16));
            }
            return Menu::Talk {
                who,
                text,
                shown,
                choices,
                sel,
                blip,
            };
        }
        let n = choices.len().max(1);
        if input.pressed_repeat(Action::Down) {
            sel = (sel + 1) % n;
            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
        }
        if input.pressed_repeat(Action::Up) {
            sel = (sel + n - 1) % n;
            io.audio.play_at(Sfx::UiMove, 0.5, 1.0);
        }
        let l = talk_layout(w, h, &text, choices.len());
        let mut picked = None;
        for i in 0..choices.len() {
            let (cx, cy) = choice_pos(&l, i);
            if inside(input.mouse, cx - 2, cy - 2, 100, 11) {
                if input.mouse_moved {
                    sel = i;
                }
                if lclick {
                    picked = Some(i);
                }
            }
        }
        if input.pressed(Action::Confirm) {
            picked = Some(sel);
        }
        if input.pressed(Action::Cancel) || input.button_pressed(Button::Right) {
            picked = choices.iter().position(|(_, s)| *s == Say::Bye);
        }
        let Some(i) = picked else {
            return Menu::Talk {
                who,
                text,
                shown,
                choices,
                sel,
                blip,
            };
        };
        io.audio.play(Sfx::UiSelect);
        let say = choices.get(i).map_or(Say::Bye, |c| c.1);
        // Choices may swap in another menu (the shop).
        self.menu = Menu::None;
        match self.talk_choice(who, say, io) {
            Some((text, choices)) if matches!(self.menu, Menu::None) => Menu::Talk {
                who,
                text,
                shown: 0.0,
                choices,
                sel: 0,
                blip: 0.0,
            },
            _ => {
                for n in &mut self.folk {
                    n.talking = false;
                }
                if matches!(self.menu, Menu::None) && self.cheer.is_some() {
                    return Menu::Cheer;
                }
                std::mem::replace(&mut self.menu, Menu::None)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_talk(
        &self,
        c: &mut Canvas,
        a: &Assets,
        who: Villager,
        text: &str,
        shown: f32,
        choices: &[(String, Say)],
        sel: usize,
    ) {
        let (w, h) = (c.w(), c.h());
        let l = talk_layout(w, h, text, choices.len());
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        // Portrait frame (the 3D portrait is drawn into it afterwards).
        let (fx, fy) = portrait_pos(&l);
        c.panel(fx - 2, fy - 2, PORTRAIT + 4, PORTRAIT + 4, Style::Inset);
        // Name plate with hearts.
        let d = who.def();
        let name = d.name;
        let nw = c.text_width(name) + c.text_width(d.title) + 22;
        c.panel(l.px + 4, l.py - 12, nw, 14, Style::Paper);
        let tx0 = l.px + 10 + c.text(l.px + 10, l.py - 9, name, RUST) + 6;
        c.text(tx0, l.py - 9, d.title, ROSEWOOD);
        let hearts = self.friends.hearts(who);
        for i in 0..10 {
            let icon = if (i as u8) < hearts {
                "heart"
            } else {
                "heart_empty"
            };
            c.sprite(
                a.tex(a.icon(icon)),
                fx - 1 + (i % 5) * 9,
                fy + PORTRAIT + 4 + (i / 5) * 8,
            );
        }
        let tx = l.px + PORTRAIT + 16;
        let tw = text_width(&l, choices.len());
        let n = shown as usize;
        let visible: String = text.chars().take(n).collect();
        let mut y = l.py + 7;
        for line in visible.split('\n') {
            let col = if line.starts_with('>') { TEAL } else { INK };
            y += c.paragraph(tx, y, tw, line, col);
        }
        let done = n >= text.chars().count();
        if done && !choices.is_empty() {
            for (i, (label, say)) in choices.iter().enumerate() {
                let (cx, cy) = choice_pos(&l, i);
                if i == sel {
                    c.rect(cx - 2, cy - 2, 100, 11, GOLD);
                    c.text(cx - 1, cy, "▶", RUST);
                }
                let col = match say {
                    Say::Offer(_) | Say::Accept(_) => CRIMSON,
                    Say::Gift => PLUM,
                    Say::Shop => TEAL,
                    Say::Spells => PURPLE,
                    _ => INK,
                };
                let short: String = label.chars().take(17).collect();
                c.text(cx + 6, cy, &short, col);
            }
        } else if !done && (self.time * 3.0).fract() < 0.6 {
            c.text(l.px + l.pw - 12, l.py + l.ph - 11, "▼", RUST);
        }
    }

    // --------------------------------------------------------------------------------------
    // Boards
    // --------------------------------------------------------------------------------------

    /// The request board (in the plaza) or the bounty board (in the guild).
    pub fn open_board(&mut self, io: &mut Io) {
        let guild = matches!(self.area, Area::Inside(town::Place::Guild));
        self.menu = Menu::Board { guild, sel: 0 };
        io.audio.play(Sfx::UiSelect);
    }

    /// Rows on a board: requests ready to hand in first, then today's notices.
    fn board_rows(&self, guild: bool) -> Vec<BoardRow> {
        let mut rows = Vec::new();
        for (i, q) in self.quests.iter().enumerate() {
            if let Some(r) = q.request() {
                if r.guild == guild {
                    rows.push(BoardRow::Open(i));
                }
            }
        }
        for r in self.notices(guild) {
            rows.push(BoardRow::Notice(r));
        }
        rows
    }

    pub fn update_board(&mut self, io: &mut Io, guild: bool, mut sel: usize) -> Menu {
        let input = io.input;
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        if input.pressed(Action::Cancel) || input.pressed(Action::Inventory) {
            io.audio.play(Sfx::UiBack);
            return Menu::None;
        }
        let rows = self.board_rows(guild);
        let n = rows.len().max(1);
        if input.pressed_repeat(Action::Down) {
            sel = (sel + 1) % n;
        }
        if input.pressed_repeat(Action::Up) {
            sel = (sel + n - 1) % n;
        }
        let l = center(w, h, 300, 196);
        let mut picked = input.pressed(Action::Confirm).then_some(sel);
        for i in 0..rows.len() {
            let y = l.py + 24 + i as i32 * 34;
            if inside(input.mouse, l.px + 6, y, l.pw - 12, 32) {
                if input.mouse_moved {
                    sel = i;
                }
                if input.button_pressed(Button::Left) {
                    picked = Some(i);
                }
            }
        }
        if let Some(i) = picked {
            match rows.get(i) {
                Some(BoardRow::Open(qi)) => {
                    if self.quest_ready(&self.quests[*qi]) {
                        self.finish_quest(*qi, io);
                        if self.cheer.is_some() {
                            return Menu::Cheer;
                        }
                    } else {
                        io.audio.play(Sfx::Denied);
                        self.toast("Not finished yet!", None, 0);
                    }
                }
                Some(BoardRow::Notice(r)) => {
                    let open = self
                        .quests
                        .iter()
                        .filter(|q| q.request().is_some_and(|x| x.guild == guild))
                        .count();
                    if open >= 5 {
                        io.audio.play(Sfx::Denied);
                        self.toast("Finish some of your open notices first.", None, 0);
                    } else {
                        self.taken.push(r.id);
                        self.accept(QuestId::Request(r.clone()), io);
                    }
                }
                None => {}
            }
        }
        Menu::Board { guild, sel }
    }

    pub fn draw_board(&self, c: &mut Canvas, a: &Assets, guild: bool, sel: usize) {
        let (w, h) = (c.w(), c.h());
        let l = center(w, h, 300, 196);
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        let title = if guild {
            "Lantern Guild Bounties"
        } else {
            "Bramblewick Request Board"
        };
        c.text_center(l.px + l.pw / 2, l.py + 7, title, RUST);
        if guild {
            let (_, rank, _) = RANKS[self.rank as usize];
            let t = format!("{rank} - {} marks", self.marks);
            c.text(
                l.px + l.pw - 8 - c.text_width(&t),
                l.py + l.ph - 13,
                &t,
                TEAL,
            );
        }
        let rows = self.board_rows(guild);
        if rows.is_empty() {
            c.text_center(
                l.px + l.pw / 2,
                l.py + 80,
                "Nothing new today. Check back tomorrow!",
                SHADOW,
            );
        }
        for (i, row) in rows.iter().enumerate() {
            let y = l.py + 24 + i as i32 * 34;
            if y + 32 > l.py + l.ph - 14 {
                break;
            }
            let hot = i == sel;
            c.panel(l.px + 6, y, l.pw - 12, 32, Style::Inset);
            if hot {
                c.frame(l.px + 6, y, l.pw - 12, 32, GOLD);
            }
            let (title, giver, goal, rewards, status) = match row {
                BoardRow::Open(qi) => {
                    let q = &self.quests[*qi];
                    let (have, need) = self.progress(q);
                    let ready = have >= need;
                    (
                        q.title(),
                        q.giver(),
                        format!("{} ({have}/{need})", goal_text(q.goal())),
                        q.rewards(),
                        if ready {
                            ("Hand in!", GREEN)
                        } else {
                            ("Taken", SHADOW)
                        },
                    )
                }
                BoardRow::Notice(r) => (
                    r.title.clone(),
                    r.giver,
                    goal_text(r.goal),
                    r.reward.clone(),
                    ("Take", RUST),
                ),
            };
            c.sprite(a.tex(a.icon("request")), l.px + 9, y + 8);
            c.text(l.px + 28, y + 3, &title, INK);
            c.text(
                l.px + l.pw - 12 - c.text_width(status.0),
                y + 3,
                status.0,
                status.1,
            );
            c.text(l.px + 28, y + 12, &goal, SHADOW);
            let mut x = l.px + 28;
            let label = format!("from {} -", giver.name());
            x += c.text(x, y + 21, &label, ROSEWOOD) + 4;
            for r in rewards.iter().take(3) {
                let t = reward_text(r);
                if x + c.text_width(&t) > l.px + l.pw - 12 {
                    break;
                }
                x += c.text(x, y + 21, &t, TEAL) + 6;
            }
        }
    }

    // --------------------------------------------------------------------------------------
    // The celebration
    // --------------------------------------------------------------------------------------

    pub fn update_cheer(&mut self, io: &mut Io) -> Menu {
        let Some(ch) = &mut self.cheer else {
            return Menu::None;
        };
        ch.t += io.dt;
        let input = io.input;
        if ch.t > 0.6
            && (input.pressed(Action::Confirm)
                || input.pressed(Action::Cancel)
                || input.button_pressed(Button::Left))
        {
            io.audio.play(Sfx::UiSelect);
            self.cheer = None;
            return Menu::None;
        }
        // Keep the confetti coming for a moment.
        if ch.t < 1.2 && (ch.t * 10.0) as i32 % 3 == 0 {
            let at = self.player.world_pos() + Vec3::new(0.0, 1.6, 0.0);
            self.fx.burst(at, 2, &[GOLD, PINK, SKY, LIME], 2.5, 2.0);
        }
        Menu::Cheer
    }

    pub fn draw_cheer(&self, c: &mut Canvas, a: &Assets) {
        let Some(ch) = &self.cheer else { return };
        let (w, h) = (c.w(), c.h());
        let rows = ch.lines.len() as i32;
        let l = center(w, h, 220, 64 + rows * 20);
        let pop = (ch.t * 6.0).min(1.0);
        let dy = ((1.0 - pop) * 20.0) as i32;
        c.panel(l.px, l.py + dy, l.pw, l.ph, Style::Paper);
        let title = if ch.title.starts_with("A gift") {
            "A present!"
        } else {
            "Quest complete!"
        };
        let tw = c.big_width(title, 2);
        let col = if (ch.t * 4.0).fract() < 0.5 {
            GOLD
        } else {
            CREAM
        };
        c.text_big(l.px + (l.pw - tw) / 2, l.py + dy + 7, title, 2, col, RUST);
        let sub = if ch.title.starts_with("A gift") {
            ch.title.clone()
        } else {
            format!("{} - for {}", ch.title, ch.who.name())
        };
        c.text_center(l.px + l.pw / 2, l.py + dy + 28, &sub, INK);
        for (i, (stack, line, color)) in ch.lines.iter().enumerate() {
            let y = l.py + dy + 42 + i as i32 * 20;
            // Rewards slide in one after another.
            if ch.t < 0.25 + i as f32 * 0.15 {
                continue;
            }
            let x = l.px + 16;
            match stack {
                Some(s) => self.draw_slot(c, a, x, y, Some(*s), false),
                None => {
                    c.panel(x, y, 18, 18, Style::Inset);
                    c.text(x + 5, y + 5, "♥", PINK);
                }
            }
            let ink = match color {
                &GOLD | &CREAM => RUST,
                &PINK => CRIMSON,
                &MINT => TEAL,
                &SKY => BLUE,
                other => {
                    if *other == WHITE {
                        INK
                    } else {
                        *other
                    }
                }
            };
            c.text(x + 24, y + 5, line, ink);
        }
        if ch.t > 0.6 && (self.time * 2.0).fract() < 0.7 {
            c.text_center(l.px + l.pw / 2, l.py + dy + l.ph - 12, "Press E", SHADOW);
        }
        // Sparkles around the box.
        for k in 0..10 {
            let a = self.time * 1.5 + k as f32 * 0.628;
            let sx = l.px + l.pw / 2 + (a.cos() * (l.pw as f32 * 0.55)) as i32;
            let sy = l.py + dy + l.ph / 2 + (a.sin() * (l.ph as f32 * 0.6)) as i32;
            c.px(sx, sy, [GOLD, WHITE, PINK, SKY][k % 4]);
        }
    }

    // --------------------------------------------------------------------------------------
    // The journal
    // --------------------------------------------------------------------------------------

    pub fn update_journal(&mut self, io: &mut Io, mut tab: usize, mut sel: usize) -> Menu {
        let input = io.input;
        let (w, h) = (io.view.0 as i32, io.view.1 as i32);
        if input.pressed(Action::Cancel) || input.pressed(Action::Quests) {
            io.audio.play(Sfx::UiBack);
            return Menu::None;
        }
        let l = super::menus::panel_layout(w, h);
        for (i, (x, tw)) in super::menus::tab_rects(&l, &JOURNAL_TABS)
            .into_iter()
            .enumerate()
        {
            if input.button_pressed(Button::Left) && inside(input.mouse, x, l.py + 5, tw, 12) {
                tab = i;
                sel = 0;
                io.audio.play(Sfx::UiMove);
            }
        }
        if input.pressed(Action::NextSlot) || input.pressed(Action::Inventory) {
            tab = (tab + 1) % 4;
            sel = 0;
        }
        if input.pressed(Action::PrevSlot) {
            tab = (tab + 3) % 4;
            sel = 0;
        }
        if tab == 3 {
            let n = FISH.len();
            if input.pressed_repeat(Action::Right) {
                sel = (sel + 1) % n;
            }
            if input.pressed_repeat(Action::Left) {
                sel = (sel + n - 1) % n;
            }
            if input.pressed_repeat(Action::Down) {
                sel = (sel + DEX_COLS).min(n - 1);
            }
            if input.pressed_repeat(Action::Up) {
                sel = sel.saturating_sub(DEX_COLS);
            }
            if input.mouse_moved {
                for i in 0..n {
                    let (x, y) = dex_cell(&l, i);
                    if inside(input.mouse, x, y, 18, 18) {
                        sel = i;
                    }
                }
            }
        }
        if tab == 0 {
            let drop_btn = (l.px + l.pw - 58, l.py + l.ph - 26, 50, 11);
            let clicked = input.button_pressed(Button::Left)
                && inside(input.mouse, drop_btn.0, drop_btn.1, drop_btn.2, drop_btn.3);
            if (clicked || input.key_pressed(crate::input::KeyCode::Delete))
                && sel < self.quests.len()
            {
                self.drop_quest(sel);
                io.audio.play(Sfx::UiBack);
                sel = sel.saturating_sub(1);
            }
            let n = self.quests.len().max(1);
            if input.pressed_repeat(Action::Down) {
                sel = (sel + 1) % n;
            }
            if input.pressed_repeat(Action::Up) {
                sel = (sel + n - 1) % n;
            }
            for i in 0..self.quests.len().min(12) {
                let y = l.py + 22 + i as i32 * 12;
                if inside(input.mouse, l.px + 6, y - 1, 104, 11) && input.mouse_moved {
                    sel = i;
                }
            }
        }
        Menu::Journal { tab, sel }
    }

    pub fn draw_journal(&self, c: &mut Canvas, a: &Assets, tab: usize, sel: usize) {
        let (w, h) = (c.w(), c.h());
        let l = super::menus::panel_layout(w, h);
        c.panel(l.px, l.py, l.pw, l.ph, Style::Paper);
        tabs(
            c,
            &l,
            &["Quests", "Friends", "Records", "Fishdex"],
            tab,
            &JOURNAL_TABS,
        );
        match tab {
            0 => {
                if self.quests.is_empty() {
                    c.text(l.px + 10, l.py + 26, "No quests yet.", SHADOW);
                    let mut y = l.py + 40;
                    for line in [
                        "Take the bus from the shelter by your",
                        "farm to Bramblewick. Everyone there has",
                        "something they'd love a hand with -",
                        "look for the ! over their heads.",
                    ] {
                        c.text(l.px + 10, y, line, INK);
                        y += 10;
                    }
                    return;
                }
                for (i, q) in self.quests.iter().enumerate().take(12) {
                    let y = l.py + 22 + i as i32 * 12;
                    let ready = self.quest_ready(q) || matches!(q.goal(), Goal::Deliver(..));
                    if i == sel {
                        c.rect(l.px + 6, y - 2, 108, 11, GOLD);
                    }
                    let mark = if q.request().is_some() { "•" } else { "★" };
                    let col = if ready { GREEN } else { INK };
                    c.text(l.px + 8, y, mark, if ready { GREEN } else { RUST });
                    let t = fit(c, &q.title(), 96);
                    c.text(l.px + 16, y, &t, col);
                }
                let Some(q) = self.quests.get(sel) else {
                    return;
                };
                let x = l.px + 116;
                let tw = l.pw - 124;
                let mut y = l.py + 22;
                c.text(x, y, &q.title(), RUST);
                y += 11;
                c.text(x, y, &format!("for {}", q.giver().name()), ROSEWOOD);
                y += 13;
                y += c.paragraph(x, y, tw, &goal_text(q.goal()), INK);
                let (have, need) = self.progress(q);
                if !matches!(q.goal(), Goal::Deliver(..)) {
                    c.bar(
                        x,
                        y + 1,
                        tw - 30,
                        6,
                        have as f32 / need.max(1) as f32,
                        GREEN,
                        LIME,
                        SHADOW,
                    );
                    c.text(x + tw - 26, y, &format!("{have}/{need}"), SHADOW);
                    y += 12;
                }
                if let Goal::Find(_, lo, hi) = q.goal() {
                    y += c.paragraph(
                        x,
                        y,
                        tw,
                        &format!("It will be on one of floors {lo}-{hi} while you look."),
                        TEAL,
                    );
                }
                if let Goal::Gather(_, _, super::quests::Source::Boss(_)) = q.goal() {
                    y += c.paragraph(x, y, tw, "The guardian returns to its floor for you.", TEAL);
                }
                y += 3;
                c.text(x, y, "Rewards", RUST);
                y += 11;
                for r in q.rewards().iter().take(5) {
                    y += c.paragraph(x + 4, y, tw - 4, &reward_text(r), TEAL);
                }
                let done_note = if q.request().is_some() {
                    "Hand in at the board."
                } else {
                    match q.goal() {
                        Goal::Deliver(_, to) => {
                            let _ = to;
                            "Talk to them to hand it over."
                        }
                        _ => "Talk to them when it's done.",
                    }
                };
                c.text(x, l.py + l.ph - 14, done_note, SHADOW);
                // Giving up is always allowed.
                let (bx, by) = (l.px + l.pw - 58, l.py + l.ph - 26);
                c.panel(bx, by, 50, 11, Style::Inset);
                c.text_center(bx + 25, by + 2, "Drop quest", ROSEWOOD);
            }
            1 => {
                c.text(
                    l.px + 10,
                    l.py + 21,
                    "Friendship: talk daily, bring gifts they love.",
                    SHADOW,
                );
                for (i, v) in VILLAGERS.iter().enumerate() {
                    let col = (i / 10) as i32;
                    let row = (i % 10) as i32;
                    let x = l.px + 8 + col * 112;
                    let y = l.py + 34 + row * 14;
                    let met = self.friends.met[*v as usize];
                    let name = if met {
                        v.def().name.rsplit(' ').next().unwrap_or("")
                    } else {
                        "???"
                    };
                    c.text(x, y, name, if met { INK } else { KHAKI });
                    let hearts = self.friends.hearts(*v);
                    for k in 0..10u8 {
                        mini_heart(c, x + 44 + k as i32 * 6, y + 2, k < hearts);
                    }
                    if self.marker(*v) == Some(false) && met {
                        c.text(x + 106, y, "!", GOLD);
                    }
                }
                let _ = (FOLK, a);
            }
            3 => self.draw_fishdex(c, a, &l, sel),
            _ => {
                let j = &self.journal;
                let dishes = super::items::ALL_ITEMS
                    .iter()
                    .filter(|i| matches!(i.def().kind, Kind::Food { .. }))
                    .count();
                let mut y = l.py + 22;
                let mut line = |c: &mut Canvas, t: String, have: usize, need: usize| {
                    c.text(l.px + 10, y, &t, INK);
                    c.bar(
                        l.px + 120,
                        y + 1,
                        70,
                        6,
                        have as f32 / need.max(1) as f32,
                        GREEN,
                        LIME,
                        SHADOW,
                    );
                    c.text(l.px + 196, y, &format!("{have}/{need}"), SHADOW);
                    y += 12;
                };
                line(c, "Crop almanac".into(), j.crops.len(), 38);
                line(c, "Curio cabinet".into(), j.curios.len(), 19);
                line(c, "Cookbook".into(), j.dishes.len(), dishes);
                line(c, "Creature codex".into(), j.foes.len(), 13);
                line(c, "Fishdex".into(), j.fish.len(), FISH.len());
                let g = j.guardians.iter().filter(|d| **d <= 60).count();
                line(c, "Guardians beaten".into(), g, 6);
                line(c, "Story quests".into(), self.done.len(), QUESTS.len());
                let (_, rank, _) = RANKS[self.rank as usize];
                c.text(
                    l.px + 10,
                    y + 2,
                    &format!("Lantern Guild: {rank} ({} marks)", self.marks),
                    TEAL,
                );
                y += 11;
                let ch = self.charisma();
                c.text(
                    l.px + 10,
                    y + 2,
                    &format!("Home: charisma {ch} ({})", super::home::charm_title(ch)),
                    PLUM,
                );
                y += 14;
                c.text(l.px + 10, y, "Bramblewick", RUST);
                y += 11;
                for (i, bit) in [
                    town::FOUNTAIN,
                    town::GARDENS,
                    town::LAMPS,
                    town::BUNTING,
                    town::BRIDGE,
                    town::CLOCK,
                    town::MARKET,
                    town::WISH_TREE,
                ]
                .into_iter()
                .enumerate()
                {
                    let x = l.px + 10 + (i % 2) as i32 * 110;
                    let yy = y + (i / 2) as i32 * 10;
                    let done = self.restored & bit != 0;
                    let mark = if done { "★" } else { "·" };
                    c.text(x, yy, mark, if done { GOLD } else { KHAKI });
                    let t: String = project_name(bit).chars().take(18).collect();
                    c.text(x + 8, yy, &t, if done { INK } else { KHAKI });
                }
            }
        }
    }

    /// Every fish there is: the ones you've caught in colour, the rest as shadows, and the
    /// details of the one picked.
    fn draw_fishdex(&self, c: &mut Canvas, a: &Assets, l: &super::menus::Layout, sel: usize) {
        let j = &self.journal;
        for (i, d) in FISH.iter().enumerate() {
            let (x, y) = dex_cell(l, i);
            let caught = j.fish.contains(&d.item);
            c.panel(x, y, 18, 18, Style::Inset);
            let icon = a.tex(a.icon(d.item.def().icon));
            if caught {
                c.sprite(icon, x + 1, y + 1);
            } else {
                c.sprite_map(icon, x + 1, y + 1, |_| ROSEWOOD);
            }
            if i == sel {
                c.frame(x - 1, y - 1, 20, 20, RUST);
            }
        }
        let rows = FISH.len().div_ceil(DEX_COLS) as i32;
        let x = l.px + 10;
        let mut y = l.py + 24 + rows * 20;
        let Some(d) = FISH.get(sel) else { return };
        let caught = j.fish.contains(&d.item);
        let ink = [SHADOW, GREEN, BLUE, RUST][d.rarity.min(3) as usize];
        let name = if caught { d.item.def().name } else { "???" };
        c.text(x, y, name, INK);
        let rare = fish::rarity_name(d.rarity);
        let rw = c.text_width(rare);
        c.text(l.px + l.pw - 10 - rw, y, rare, ink);
        y += 11;
        let waters: Vec<&str> = d.water.iter().map(|w| w.short()).collect();
        c.text(x, y, &format!("Lives in: {}", waters.join(", ")), TEAL);
        y += 10;
        c.text(x, y, d.when.label(), ROSEWOOD);
        y += 10;
        let record = j.records.iter().find(|(i, _)| *i == d.item).map(|r| r.1);
        let t = match record {
            Some(cm) => format!("Your biggest: {cm} cm  (up to {} cm)", d.size.1),
            None => "Not caught yet.".to_string(),
        };
        c.text(x, y, &t, if record.is_some() { INK } else { KHAKI });
        c.text(
            x,
            l.py + l.ph - 14,
            &format!(
                "{}/{} kinds   {} caught   {} cooked",
                j.fish.len(),
                FISH.len(),
                self.stats.caught,
                self.stats.cooked
            ),
            SHADOW,
        );
    }

    // --------------------------------------------------------------------------------------
    // Heads-up bits
    // --------------------------------------------------------------------------------------

    /// Names over nearby villagers, and the quest tracker.
    pub fn draw_folk_hud(&self, c: &mut Canvas, cam: &Camera) {
        for n in &self.folk {
            let d = (n.pos - self.player.pos).length();
            if d > 2.6 {
                continue;
            }
            let scale = n.who.def().look.scale;
            if let Some(s) = cam.project(n.world_pos() + Vec3::Y * (1.25 * scale + 0.25)) {
                let name = n.who.def().name;
                let tw = c.text_width(name);
                c.text_outline(s.x as i32 - tw / 2, s.y as i32 - 12, name, CREAM, INK);
            }
        }
    }

    pub fn draw_tracker(&self, c: &mut Canvas, a: &Assets, y0: i32) {
        let mut rows: Vec<(String, String, bool)> = Vec::new();
        for q in self.quests.iter().take(3) {
            let (have, need) = self.progress(q);
            let ready = have >= need;
            let status = match q.goal() {
                Goal::Deliver(_, to) => format!("to {}", to.name()),
                _ if ready => "done!".to_string(),
                _ => format!("{have}/{need}"),
            };
            rows.push((q.title(), status, ready));
        }
        if let Area::Hollow { depth } = self.area {
            if !self.keepsakes_here(depth).is_empty() {
                rows.push(("A keepsake is on".into(), "this floor!".into(), true));
            }
        }
        if rows.is_empty() {
            return;
        }
        let pw = 124;
        let ph = 12 + rows.len() as i32 * 10;
        c.panel(3, y0, pw, ph, Style::Dark);
        c.text_shadow(7, y0 + 2, "★ Quests  (L)", GOLD, INK);
        for (i, (t, s, ready)) in rows.iter().enumerate() {
            let y = y0 + 12 + i as i32 * 10;
            let sw = c.text_width(s);
            let t = fit(c, t, pw - 14 - sw);
            c.text_shadow(7, y, &t, if *ready { LIME } else { CREAM }, INK);
            c.text_shadow(pw - 3 - sw, y, s, if *ready { LIME } else { SKY }, INK);
        }
        let _ = a;
    }
}

/// A five-pixel heart for the friends list.
fn mini_heart(c: &mut Canvas, x: i32, y: i32, full: bool) {
    const ROWS: [&str; 5] = [".o.o.", "ooooo", "ooooo", ".ooo.", "..o.."];
    for (dy, r) in ROWS.iter().enumerate() {
        for (dx, ch) in r.chars().enumerate() {
            if ch == 'o' {
                let col = if full {
                    if dy == 1 && dx == 1 { BLUSH } else { CRIMSON }
                } else {
                    KHAKI
                };
                c.px(x + dx as i32, y + dy as i32, col);
            }
        }
    }
}

/// Cuts text to fit a width, with an ellipsis.
pub fn fit(c: &Canvas, s: &str, w: i32) -> String {
    if c.text_width(s) <= w {
        return s.to_string();
    }
    let mut out: String = s.to_string();
    while !out.is_empty() && c.text_width(&format!("{out}…")) > w {
        out.pop();
    }
    format!("{}…", out.trim_end())
}

enum BoardRow {
    Open(usize),
    Notice(super::quests::Request),
}

pub const JOURNAL_TABS: [i32; 4] = [44, 46, 48, 48];

/// Fish per row in the Fishdex.
const DEX_COLS: usize = 11;

/// Where a fish's box is in the Fishdex.
fn dex_cell(l: &super::menus::Layout, i: usize) -> (i32, i32) {
    (
        l.px + 8 + (i % DEX_COLS) as i32 * 20,
        l.py + 21 + (i / DEX_COLS) as i32 * 20,
    )
}

pub const PORTRAIT: i32 = 44;

pub struct TalkLayout {
    pub px: i32,
    pub py: i32,
    pub pw: i32,
    pub ph: i32,
}

fn text_width(l: &TalkLayout, choices: usize) -> i32 {
    let right = if choices > 0 { 108 } else { 8 };
    l.pw - PORTRAIT - 24 - right
}

pub fn talk_layout(w: i32, h: i32, text: &str, choices: usize) -> TalkLayout {
    let pw = (w - 24).min(400);
    let probe = TalkLayout {
        px: 0,
        py: 0,
        pw,
        ph: 0,
    };
    let tw = text_width(&probe, choices);
    // A rough line count: the font is about five pixels a letter.
    let per = (tw / 5).max(10) as usize;
    let lines: usize = text.split('\n').map(|p| p.chars().count() / per + 1).sum();
    let ph = (16 + lines as i32 * 10)
        .max(choices as i32 * 12 + 12)
        .max(PORTRAIT + 30);
    TalkLayout {
        px: (w - pw) / 2,
        py: h - 22 - ph,
        pw,
        ph,
    }
}

pub fn portrait_pos(l: &TalkLayout) -> (i32, i32) {
    (l.px + 8, l.py + 8)
}

fn choice_pos(l: &TalkLayout, i: usize) -> (i32, i32) {
    (l.px + l.pw - 104, l.py + 8 + i as i32 * 12)
}

/// The size of the face in the talk box, in pixels.
pub fn portrait_rect(w: i32, h: i32, menu: &Menu) -> Option<(Villager, i32, i32)> {
    match menu {
        Menu::Talk {
            who, text, choices, ..
        } => {
            let l = talk_layout(w, h, text, choices.len());
            let (x, y) = portrait_pos(&l);
            Some((*who, x, y))
        }
        _ => None,
    }
}

/// Where you are standing relative to someone: used for the talk prompt.
pub fn near_text(v: Villager) -> String {
    format!("Talk to {}", v.def().name)
}

#[allow(dead_code)]
fn _unused(_: Vec2) {}
