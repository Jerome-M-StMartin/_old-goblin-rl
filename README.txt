PROSPECTIVE EMPLOYERS:
This part is just for you. I'm just about to link this repo to a recruiter and
wanted to clarify which parts of this source code are worth your time. If
you'd like to see my most recent original, compeltely written-from-scratch
work, please navigate to the following:
../src/command/mod.rs
../src/user_input/mod.rs
../src/gui/observer/mod.rs
../src/gui/widget/widget_builder.rs

In these locations you will see implementations of the following OOP design
patterns: Command Pattern, Observer Pattern, and Builder Pattern.
user_input/mod.rs is the Observable side of the Observer pattern, while
widget_builder.rs implements all three, Observor, Commandable, and Builder.

============================================================================

Alpha Title: GoblinRL
Alt. Titles: ????:?? / ??????RL

Description: A roguelike, written in Rust, based on the work of those who
came before as all things are. Shoutout to bracket-lib, specs, serde, and
code goblins everywhere.

Snippet of Planned Gameplay Features:
- Robust Stealth Logic
- Context Menu for any/every in-diegesis thing.
- Conlang-based spellcasting/ability usage.
- HP as a spendable resource.

Began May, 2020; temporary hiatus for several months;
in progress as of Feb, 2022 + (see latest commit).
