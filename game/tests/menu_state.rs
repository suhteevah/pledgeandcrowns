// SPDX-License-Identifier: MIT
//! Pure rules behind the state machine and menus: Esc routing, menu
//! composition, selection wrap, slider nudges and the volume curve.
//! Plus a headless App check that pausing keeps `GameState::InGame`.

use bevy::prelude::*;
use bevy::state::app::StatesPlugin;
use bevy_kira_audio::prelude::Decibels;
use pledge_and_crown::plugins::audio::volume_db;
use pledge_and_crown::plugins::menu::{PauseItem, SLIDER_STEP, nudge_volume, step_selection};
use pledge_and_crown::plugins::state::{
    EscAction, EscContext, GameState, PauseState, StatePlugin, esc_action,
};
use pledge_and_crown::plugins::title::{TitleItem, title_items};

fn ctx(state: GameState, paused: bool) -> EscContext {
    EscContext {
        state,
        paused,
        editor_open: false,
        overlay_open: false,
        submenu_open: false,
    }
}

#[test]
fn esc_pauses_running_game() {
    assert_eq!(esc_action(ctx(GameState::InGame, false)), EscAction::Pause);
}

#[test]
fn esc_ignored_while_editor_open() {
    let c = EscContext {
        editor_open: true,
        ..ctx(GameState::InGame, false)
    };
    assert_eq!(esc_action(c), EscAction::Ignore);
}

#[test]
fn esc_ignored_while_overlay_open() {
    let c = EscContext {
        overlay_open: true,
        ..ctx(GameState::InGame, false)
    };
    assert_eq!(esc_action(c), EscAction::Ignore);
}

#[test]
fn esc_resumes_when_paused() {
    assert_eq!(esc_action(ctx(GameState::InGame, true)), EscAction::Resume);
}

#[test]
fn esc_backs_out_of_pause_submenu_before_resuming() {
    let c = EscContext {
        submenu_open: true,
        ..ctx(GameState::InGame, true)
    };
    assert_eq!(esc_action(c), EscAction::CloseSubmenu);
}

#[test]
fn esc_on_title() {
    assert_eq!(esc_action(ctx(GameState::Title, false)), EscAction::Ignore);
    let c = EscContext {
        submenu_open: true,
        ..ctx(GameState::Title, false)
    };
    assert_eq!(esc_action(c), EscAction::CloseSubmenu);
}

#[test]
fn title_items_continue_first_only_with_progress() {
    assert_eq!(
        title_items(true, true),
        vec![
            TitleItem::Continue,
            TitleItem::NewGame,
            TitleItem::Settings,
            TitleItem::Quit
        ]
    );
    // Default highlight (index 0) is New Game when there is nothing to continue.
    assert_eq!(title_items(false, true)[0], TitleItem::NewGame);
    assert!(!title_items(false, true).contains(&TitleItem::Continue));
}

#[test]
fn title_items_hide_quit_when_unavailable() {
    assert!(!title_items(true, false).contains(&TitleItem::Quit));
}

#[test]
fn pause_items_order() {
    assert_eq!(
        PauseItem::ALL,
        [
            PauseItem::Resume,
            PauseItem::Settings,
            PauseItem::QuitToTitle
        ]
    );
}

#[test]
fn selection_wraps_both_ways() {
    assert_eq!(step_selection(0, 3, -1), 2);
    assert_eq!(step_selection(2, 3, 1), 0);
    assert_eq!(step_selection(1, 3, 0), 1);
    assert_eq!(step_selection(7, 3, 0), 2, "stale index clamps into range");
    assert_eq!(step_selection(0, 0, 1), 0);
}

#[test]
fn nudge_saturates() {
    assert_eq!(nudge_volume(50, 1), 50 + SLIDER_STEP);
    assert_eq!(nudge_volume(98, 1), 100);
    assert_eq!(nudge_volume(3, -1), 0);
}

#[test]
fn volume_mapping() {
    let silence = Decibels::SILENCE.0;
    assert!(volume_db(100, 100).abs() < 1e-4, "full volume is 0 dB");
    assert_eq!(volume_db(0, 100), silence);
    assert_eq!(volume_db(100, 0), silence);
    // Half amplitude is about -6.02 dB, regardless of which slider halves it.
    assert!((volume_db(50, 100) - (-6.0206)).abs() < 1e-3);
    assert!((volume_db(100, 50) - volume_db(50, 100)).abs() < 1e-6);
    // Master and channel multiply: 50% x 50% = 25% amplitude, about -12 dB.
    assert!((volume_db(50, 50) - (-12.0412)).abs() < 1e-3);
    // Tiny gains floor at silence rather than going to -inf.
    assert_eq!(volume_db(1, 1), (20.0f32 * 0.0001f32.log10()).max(silence));
    // Out-of-range input is clamped, never louder than 0 dB.
    assert!(volume_db(255, 255) <= 1e-4);
    // Monotonic in each slider.
    let mut prev = silence;
    for v in 1..=100u8 {
        let db = volume_db(100, v);
        assert!(db >= prev, "not monotonic at {v}");
        prev = db;
    }
}

#[test]
fn pausing_keeps_game_state_in_game() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .add_plugins(StatePlugin);
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::InGame);
    app.update();
    assert_eq!(
        *app.world().resource::<State<PauseState>>().get(),
        PauseState::Running
    );

    app.world_mut()
        .resource_mut::<NextState<PauseState>>()
        .set(PauseState::Paused);
    app.update();
    assert_eq!(
        *app.world().resource::<State<GameState>>().get(),
        GameState::InGame,
        "pause must not leave InGame (that would respawn the world)"
    );
    assert!(app.world().resource::<Time<Virtual>>().is_paused());

    app.world_mut()
        .resource_mut::<NextState<PauseState>>()
        .set(PauseState::Running);
    app.update();
    assert!(!app.world().resource::<Time<Virtual>>().is_paused());

    // Back to title removes the pause sub-state entirely.
    app.world_mut()
        .resource_mut::<NextState<PauseState>>()
        .set(PauseState::Paused);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Title);
    app.update();
    assert!(app.world().get_resource::<State<PauseState>>().is_none());
    assert!(
        !app.world().resource::<Time<Virtual>>().is_paused(),
        "leaving InGame while paused must thaw time"
    );
}
