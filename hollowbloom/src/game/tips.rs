//! Shared UI pieces: item slots with rarity frames, coin amounts, and the tooltips that
//! explain gear, scrolls, food and everything else.

use super::gear::{Group, Rarity, SOCKETS, Stat};
use super::items::{Kind, Stack};
use super::loot::money_parts;
use super::play::Play;
use crate::assets::Assets;
use crate::palette::*;
use crate::ui::{Canvas, Style};

/// Width of a coin amount drawn by [`draw_money`].
pub fn money_width(c: &Canvas, amount: u64) -> i32 {
    let (g, s, cc) = money_parts(amount);
    let mut w = 0;
    let mut any = false;
    for (n, show) in [(g, g > 0), (s, s > 0), (cc, cc > 0 || (g == 0 && s == 0))] {
        if show {
            if any {
                w += 3;
            }
            w += 9 + c.text_width(&crate::util::thousands(n));
            any = true;
        }
    }
    w
}

/// Draws an amount as gold, silver and copper coins: "●12 ●3 ●4".
pub fn draw_money(c: &mut Canvas, a: &Assets, x: i32, y: i32, amount: u64, color: u8) -> i32 {
    let (g, s, cc) = money_parts(amount);
    let mut cx = x;
    let mut any = false;
    for (n, icon, show) in [
        (g, "coin_gold", g > 0),
        (s, "coin_silver", s > 0),
        (cc, "coin_copper", cc > 0 || (g == 0 && s == 0)),
    ] {
        if !show {
            continue;
        }
        if any {
            cx += 3;
        }
        c.sprite(a.tex(a.icon(icon)), cx, y);
        cx += 9;
        cx += c.text(cx, y, &crate::util::thousands(n), color) + 1;
        any = true;
    }
    cx - x
}

/// A dark tooltip box with a border in the given colour.
pub(super) fn tip_box(c: &mut Canvas, x: i32, y: i32, w: i32, h: i32, border: u8) {
    c.rect(x + 1, y + 1, w - 2, h - 2, INK);
    c.rect(x + 1, y, w - 2, 1, border);
    c.rect(x + 1, y + h - 1, w - 2, 1, border);
    c.rect(x, y + 1, 1, h - 2, border);
    c.rect(x + w - 1, y + 1, 1, h - 2, border);
    c.rect(x + 2, y + 2, w - 4, 1, SHADOW);
}

/// A line of tooltip text with an optional small icon in front.
pub(super) struct Line {
    pub text: String,
    pub color: u8,
    pub icon: Option<&'static str>,
    pub gap: bool,
}

pub(super) fn line(text: impl Into<String>, color: u8) -> Line {
    Line {
        text: text.into(),
        color,
        icon: None,
        gap: false,
    }
}

impl Play {
    /// One inventory slot: the item, its count, and a frame in its rarity's colour.
    pub fn draw_slot(
        &self,
        c: &mut Canvas,
        a: &Assets,
        x: i32,
        y: i32,
        s: Option<Stack>,
        selected: bool,
    ) {
        c.panel(x, y, 18, 18, Style::Inset);
        if let Some(s) = s {
            if let Some(r) = s.rarity().filter(|r| *r > Rarity::Common) {
                c.frame(x + 1, y + 1, 16, 16, r.shade());
                c.px(x + 1, y + 1, r.color());
                c.px(x + 2, y + 1, r.color());
                c.px(x + 1, y + 2, r.color());
                if r == Rarity::Legendary {
                    // A glint running round the frame.
                    let k = ((self.time * 24.0) as i32).rem_euclid(60);
                    let (gx, gy) = match k {
                        0..15 => (x + 1 + k, y + 1),
                        15..30 => (x + 16, y + 1 + (k - 15)),
                        30..45 => (x + 16 - (k - 30), y + 16),
                        _ => (x + 1, y + 16 - (k - 45)),
                    };
                    c.px(gx, gy, WHITE);
                }
            }
            c.sprite(a.tex(a.stack_icon(&s)), x + 1, y + 1);
            if s.n > 1 {
                c.tiny(x + 17, y + 12, &s.n.to_string(), WHITE, INK);
            }
            // Forged armour wears its level in the corner.
            if let Some(f) = s.gear.map(|g| g.forge()).filter(|f| *f > 0) {
                c.tiny(x + 17, y + 12, &format!("+{f}"), GOLD, INK);
            }
        }
        if selected {
            c.frame(x - 1, y - 1, 20, 20, ORANGE);
            c.frame(x, y, 18, 18, GOLD);
        }
    }

    /// The tooltip for a stack, placed near (x, y) and kept on screen.
    pub fn stack_tooltip(&self, c: &mut Canvas, a: &Assets, x: i32, y: i32, s: &Stack) {
        let d = s.item.def();
        let title_color = s.rarity().map_or(WHITE, |r| r.color());
        let lines = self.stack_lines(s);
        self.tooltip_box(c, a, x, y, s, &lines, d.desc, title_color);
    }

    /// What a stack is and does, line by line: its kind, what eating or drinking it gives
    /// (buffs and all), a piece of gear's stats, a piece of furniture's charm.
    pub(super) fn stack_lines(&self, s: &Stack) -> Vec<Line> {
        let d = s.item.def();
        let mut lines: Vec<Line> = Vec::new();
        match d.kind {
            Kind::Gear(b) => {
                let g = s.gear.unwrap_or_else(|| super::gear::Gear::plain(b.lvl));
                lines.push(line(
                    format!("{} {} - Lv {}", g.rarity.name(), b.class.name(), g.level),
                    KHAKI,
                ));
                // How far it's been forged on the anvil.
                if b.class.is_armor() && g.xp > 0 {
                    let text = match g.forge_progress() {
                        (_, None) => format!("Forged +{} - as strong as it gets", g.forge()),
                        (into, Some(need)) => {
                            format!("Forged +{} - {into}/{need} forge xp", g.forge())
                        }
                    };
                    lines.push(line(text, GOLD));
                }
                let main = s.main_value().unwrap_or(0);
                let mut ml = match b.class {
                    super::gear::Class::Hoe => {
                        let n = main;
                        line(
                            format!("Tills {n} tile{}", if n == 1 { "" } else { "s" }),
                            WHITE,
                        )
                    }
                    super::gear::Class::Can => line(format!("Holds {main} water"), WHITE),
                    super::gear::Class::Wand => {
                        line(format!("{main} Magic Damage - 5 mana"), WHITE)
                    }
                    super::gear::Class::Staff => {
                        line(format!("{main} Blast Damage - 12 mana"), WHITE)
                    }
                    super::gear::Class::Shield => {
                        let block = 8 + (g.level as i32 / 6).min(12);
                        line(format!("{main} Defense - {block}% Block"), WHITE)
                    }
                    _ => line(format!("{main} {}", b.class.main_label()), WHITE),
                };
                ml.gap = true;
                // Compare armour with what is worn.
                if let Some(slot) = b.class.slot() {
                    if let Some(worn) = self.player.worn(slot) {
                        if worn != s {
                            let diff = main - worn.main_value().unwrap_or(0);
                            if diff != 0 {
                                ml.text = format!(
                                    "{}  ({}{} worn)",
                                    ml.text,
                                    if diff > 0 { "+" } else { "" },
                                    diff
                                );
                                ml.color = if diff > 0 { LIME } else { SALMON };
                            }
                        }
                    }
                }
                lines.push(ml);
                if let Some((st, v)) = s.innate() {
                    let mut l = line(format!("{} (innate)", st.line(v)), CREAM);
                    l.icon = Some(stat_icon(st));
                    lines.push(l);
                }
                for af in g.affixes() {
                    let v = g.forged_stat(af.val as i32);
                    let mut l = line(af.stat.line(v), af.stat.def().color);
                    l.icon = Some(stat_icon(af.stat));
                    lines.push(l);
                }
                let mut first = true;
                for e in g.enchants.iter() {
                    let mut l = match e {
                        Some(e) => {
                            let v = g.forged_stat(e.val as i32);
                            let mut l = line(format!("{} *", e.stat.line(v)), BLUSH);
                            l.icon = Some(stat_icon(e.stat));
                            l
                        }
                        None => {
                            let mut l = line("Empty enchant socket", SHADOW);
                            l.icon = Some("socket");
                            l
                        }
                    };
                    l.gap = first;
                    first = false;
                    lines.push(l);
                }
            }
            Kind::Scroll(group) => {
                let g = s.gear.unwrap_or_else(|| super::gear::Gear::plain(1));
                lines.push(line(
                    format!(
                        "{} {} Scroll - Lv {}",
                        g.rarity.name(),
                        group.name(),
                        g.level
                    ),
                    KHAKI,
                ));
                if let Some(e) = g.scroll_enchant() {
                    let mut l = line(e.stat.line(e.val as i32), e.stat.def().color);
                    l.icon = Some(stat_icon(e.stat));
                    l.gap = true;
                    lines.push(l);
                    lines.push(line(e.stat.def().about, KHAKI));
                    let suits: Vec<&str> = super::gear::CLASSES
                        .iter()
                        .filter(|cl| cl.group() == group && cl.suits(e.stat))
                        .map(|cl| cl.plural())
                        .collect();
                    lines.push(line(format!("For {}", suits.join(", ")), SKY));
                }
                let _ = SOCKETS;
            }
            Kind::Seed(crop) => {
                let cd = crop.def();
                lines.push(line(
                    if cd.regrow > 0 {
                        format!("Seed - {} days, then every {}", cd.days, cd.regrow)
                    } else {
                        format!("Seed - {} days", cd.days)
                    },
                    LIME,
                ));
            }
            Kind::Produce { hp, energy } => {
                lines.push(line(format!("Food - +{hp} HP  +{energy} energy"), LIME));
            }
            Kind::Food {
                hp,
                energy,
                mana,
                buff,
            } => {
                let mut t = String::from("Food -");
                if hp > 0 {
                    t += &format!(" +{hp} HP");
                }
                if energy > 0 {
                    t += &format!(" +{energy} energy");
                }
                if mana > 0 {
                    t += &format!(" +{mana} mana");
                }
                lines.push(line(t, LIME));
                if let Some(b) = buff {
                    let mut l = line(
                        format!("{} for {}", b.stat.line(b.val as i32), duration(b.secs)),
                        b.stat.def().color,
                    );
                    l.icon = Some(stat_icon(b.stat));
                    lines.push(l);
                }
            }
            Kind::Potion { hp, mana, energy } => {
                let mut t = String::from("Potion -");
                if hp > 0 {
                    t += &format!(" +{hp} HP");
                }
                if mana > 0 {
                    t += &format!(" +{mana} mana");
                }
                if energy > 0 {
                    t += &format!(" +{energy} energy");
                }
                lines.push(line(t, PINK));
            }
            Kind::Coin(_) => lines.push(line("Money", GOLD)),
            Kind::Gem => lines.push(line("Gem", AQUA)),
            Kind::Relic => lines.push(line("Treasure - Burrowby loves these", GOLD)),
            Kind::Fish => {
                if let Some(f) = super::fish::fish_def(s.item) {
                    lines.push(line(
                        format!("{} Fish", super::fish::rarity_name(f.rarity)),
                        super::fish::rarity_color(f.rarity),
                    ));
                    for w in f.water {
                        lines.push(line(format!("Lives in the {}", w.short()), SKY));
                    }
                    lines.push(line(f.when.label(), KHAKI));
                    if let Some((_, cm)) = self.journal.records.iter().find(|(i, _)| *i == s.item) {
                        lines.push(line(format!("Your biggest: {cm} cm"), GOLD));
                    }
                }
            }
            Kind::Place(super::items::Placeable::Furniture(f)) => {
                let fd = f.def();
                lines.push(line(
                    format!("Furniture - {}x{} tiles", fd.size.0, fd.size.1),
                    SKY,
                ));
                lines.push(line(format!("Charm +{}", fd.charm), PINK));
                if fd.cap > 0 {
                    lines.push(line(format!("Holds {} fish", fd.cap), AQUA));
                }
                if fd.use_ != super::home::Use::Nothing {
                    lines.push(line(format!("Use: {}", super::home::use_hint(f)), KHAKI));
                }
            }
            Kind::Place(super::items::Placeable::Rug(k)) => {
                lines.push(line("Rug - covers 2x2 tiles", SKY));
                let charm = super::home::RUGS[k as usize % super::home::RUGS.len()].0;
                lines.push(line(format!("Charm +{charm}"), PINK));
            }
            Kind::Place(super::items::Placeable::WallArt(k)) => {
                lines.push(line("Wall art - for the back wall", SKY));
                let charm = super::home::ART[k as usize % super::home::ART.len()];
                lines.push(line(format!("Charm +{charm}"), PINK));
            }
            Kind::Wallpaper(_) | Kind::Flooring(_) => {
                let what = if matches!(d.kind, Kind::Wallpaper(_)) {
                    "Wallpaper"
                } else {
                    "Flooring"
                };
                lines.push(line(format!("{what} - use it in your house"), SKY));
                lines.push(line("Charm +3", PINK));
            }
            Kind::Place(_) => lines.push(line("Placeable", SKY)),
            Kind::Material => lines.push(line("Material", KHAKI)),
            Kind::Bomb => {
                lines.push(line(
                    format!(
                        "Throw it ({}) - goes off in 2 seconds",
                        self.key(crate::input::Action::Use)
                    ),
                    ORANGE,
                ));
                lines.push(line("Opens cracked floors in the Hollow", GOLD));
            }
            Kind::InkMap => {
                lines.push(line(
                    format!(
                        "Read it ({}) in the Hollow",
                        self.key(crate::input::Action::Interact)
                    ),
                    AQUA,
                ));
                lines.push(line("Glowing runes show the way down", MINT));
            }
            Kind::Pack { .. } => {
                let p = s.pack.unwrap_or(super::items::Pack { slots: 0, hue: 0 });
                lines.push(line(format!("{} Backpack", p.rarity().name()), KHAKI));
                let mut room = line(format!("+{} bag slots", p.slots), WHITE);
                room.gap = true;
                // Compare with the one on your back.
                if let Some(worn) = self.player.pack.filter(|w| w != s) {
                    let diff = p.slots as i32 - worn.pack_slots() as i32;
                    if diff != 0 {
                        room.text = format!(
                            "{}  ({}{} worn)",
                            room.text,
                            if diff > 0 { "+" } else { "" },
                            diff
                        );
                        room.color = if diff > 0 { LIME } else { SALMON };
                    }
                }
                lines.push(room);
                lines.push(line("Wear it in the backpack slot", SKY));
            }
            _ => lines.push(line("Special", LAVENDER)),
        }
        lines
    }

    #[allow(clippy::too_many_arguments)]
    fn tooltip_box(
        &self,
        c: &mut Canvas,
        a: &Assets,
        x: i32,
        y: i32,
        s: &Stack,
        lines: &[Line],
        about: &str,
        title_color: u8,
    ) {
        let wrap_w = 150;
        let desc = c.font.wrap(about, wrap_w);
        let name = s.name();
        let price = if super::menus::can_sell(s) {
            Some(s.unit_price())
        } else {
            None
        };
        let inner = lines
            .iter()
            .map(|l| c.text_width(&l.text) + if l.icon.is_some() { 10 } else { 0 })
            .chain(desc.iter().map(|l| c.text_width(l)))
            .chain(std::iter::once(c.text_width(&name)))
            .chain(price.map(|p| 58 + money_width(c, p)))
            .max()
            .unwrap_or(40);
        let w = inner.max(60) + 12;
        let gaps = lines.iter().filter(|l| l.gap).count() as i32;
        let h = 17
            + lines.len() as i32 * 10
            + gaps * 3
            + desc.len() as i32 * 9
            + 5
            + if price.is_some() { 12 } else { 0 };
        let x = x.min(c.w() - w - 2).max(2);
        let y = y.min(c.h() - h - 2).max(2);
        let border = s.rarity().map_or(KHAKI, |r| r.shade());
        tip_box(c, x, y, w, h, border);
        c.text_shadow(x + 6, y + 4, &name, title_color, SHADOW);
        let mut yy = y + 16;
        for l in lines {
            if l.gap {
                c.rect(x + 5, yy + 1, w - 10, 1, SHADOW);
                yy += 3;
            }
            let mut tx = x + 6;
            if let Some(icon) = l.icon {
                c.sprite(a.tex(a.icon(icon)), tx, yy);
                tx += 10;
            }
            c.text(tx, yy, &l.text, l.color);
            yy += 10;
        }
        yy += 2;
        for l in &desc {
            c.text(x + 6, yy, l, KHAKI);
            yy += 9;
        }
        if let Some(p) = price {
            yy += 2;
            let lw = c.text(x + 6, yy, "Sells for", ROSEWOOD);
            draw_money(c, a, x + 10 + lw, yy, p, CREAM);
        }
    }
}

/// "2 min", "45 s".
pub fn duration(secs: u16) -> String {
    if secs >= 60 && secs % 60 == 0 {
        format!("{} min", secs / 60)
    } else if secs >= 60 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{secs} s")
    }
}

/// The small picture for a stat.
pub fn stat_icon(s: Stat) -> &'static str {
    use Stat::*;
    match s {
        Damage => "st_damage",
        Crit => "st_crit",
        CritDmg => "st_critdmg",
        Haste => "st_haste",
        Lifesteal => "st_lifesteal",
        Burn => "st_burn",
        Chill => "st_chill",
        Shock => "st_shock",
        Defense => "st_defense",
        Vitality => "st_vitality",
        Stamina => "st_stamina",
        Wisdom => "st_wisdom",
        Regen => "st_regen",
        Swift => "st_swift",
        Dodge => "st_dodge",
        Block => "st_block",
        Thorns => "st_thorns",
        Luck => "st_luck",
        Greed => "st_greed",
        Power => "st_power",
        Frugal => "st_frugal",
        Reach => "st_reach",
        Capacity => "st_capacity",
        Bounty => "st_bounty",
        Growth => "st_growth",
        Forage => "st_forage",
        Focus => "st_focus",
        Spirit => "st_spirit",
        Lure => "st_lure",
        Line => "st_line",
        Treasure => "st_treasure",
        Angler => "st_angler",
    }
}

/// The scroll icon for a group.
pub fn scroll_group_name(g: Group) -> &'static str {
    match g {
        Group::Weapon => "swords, wands and staffs",
        Group::Armor => "shields, hats, armor and boots",
        Group::Tool => "hoes, cans, sickles, axes, pickaxes and rods",
    }
}
