mod debug;
mod frame_per_second;

use bevy::{input::mouse::MouseButtonInput, prelude::*, winit::WinitSettings};
use debug::DebugPlugin;
use frame_per_second::FPSDiagPlugin;

const CLEAR: Color = Color::rgb(0.1, 0.1, 0.1);
const RESOLUTION: f32 = 1.0 / 1.0;
const HEIGHT: f32 = 800.0;
const WHITE_SQUARE_COLOR: Color = Color::rgb(240. / 255., 217. / 255., 181. / 255.);
const BLACK_SQUARE_COLOR: Color = Color::rgb(181. / 255., 136. / 255., 99. / 255.);
const START_COLOR: Color = Color::rgb(0.35, 0.75, 0.35);
const END_COLOR: Color = Color::rgb(0.8, 0.75, 0.35);

// TIMERS
struct DebugTimer(Timer);

struct SelectTimer(Timer);

#[derive(Debug, Component)]
struct BoardComponent {
    board: chess::Board,
}

#[derive(Debug, Component)]
struct PieceComponent {
    position: Vec2,
}

#[derive(Debug, Clone, Component)]
struct SquareComponent {
    chess_sq: chess::Square,
    position: Vec2,
    bottom_left_coord: Vec2,
    piece_size: f32,
}

#[derive(Debug, Component)]
struct SelectingSquares {
    start: Option<SquareComponent>,
    end: Option<SquareComponent>,
}

#[derive(Debug, Component)]
struct SelectingStartSquare;

#[derive(Debug, Component)]
struct SelectingEndSquare;

fn main() {
    App::new()
        .add_startup_system(spawn_camera)
        .add_startup_system_to_stage(StartupStage::PreStartup, load_chess_piece_sprites)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_pieces)
        .insert_resource(ClearColor(CLEAR))
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(WindowDescriptor {
            width: HEIGHT * RESOLUTION,
            height: HEIGHT,
            title: "Bevy chess by Chop Tr".to_string(),
            resizable: false,
            ..Default::default()
        })
        .insert_resource(DebugTimer(Timer::from_seconds(2.0, true)))
        .insert_resource(SelectTimer(Timer::from_seconds(2.0, false)))
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugPlugin)
        .add_plugin(FPSDiagPlugin)
        .add_system(mouse_select_system)
        .add_system(highlight_selected)
        .add_system(handle_chess_move)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn_bundle(Camera2dBundle::default());
}

struct ChessPieceSprites(Handle<TextureAtlas>);

fn load_chess_piece_sprites(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let image = asset_server.load("chess-pieces.png");
    let texture_atlas = TextureAtlas::from_grid(image, Vec2::new(106., 106.), 6, 2);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    commands.insert_resource(ChessPieceSprites(texture_atlas_handle));
}

#[derive(Copy, Clone)]
enum PieceSprite {
    WhiteKing = 0,
    WhiteQueen,
    WhiteBishop,
    WhiteKnight,
    WhiteRook,
    WhitePawn,
    BlackKing,
    BlackQueen,
    BlackBishop,
    BlackKnight,
    BlackRook,
    BlackPawn,
}

impl PieceSprite {
    fn from_chess(pc: chess::Piece, color: chess::Color) -> Self {
        match (pc, color) {
            (chess::Piece::King, chess::Color::White) => Self::WhiteKing,
            (chess::Piece::Queen, chess::Color::White) => Self::WhiteQueen,
            (chess::Piece::Bishop, chess::Color::White) => Self::WhiteBishop,
            (chess::Piece::Knight, chess::Color::White) => Self::WhiteKnight,
            (chess::Piece::Rook, chess::Color::White) => Self::WhiteRook,
            (chess::Piece::Pawn, chess::Color::White) => Self::WhitePawn,
            (chess::Piece::King, chess::Color::Black) => Self::BlackKing,
            (chess::Piece::Queen, chess::Color::Black) => Self::BlackQueen,
            (chess::Piece::Bishop, chess::Color::Black) => Self::BlackBishop,
            (chess::Piece::Knight, chess::Color::Black) => Self::BlackKnight,
            (chess::Piece::Rook, chess::Color::Black) => Self::BlackRook,
            (chess::Piece::Pawn, chess::Color::Black) => Self::BlackPawn,
        }
    }
}

fn translate_square_to_xy(sq: chess::Square) -> (usize, usize) {
    let sq_index = sq.to_index();
    let x = sq_index % 8;
    let y = sq_index / 8;
    (x, y)
}

fn translate_xy_to_center_coord(x: usize, y: usize) -> (i32, i32) {
    let delta = 4;
    let new_x = x as i32 - delta;
    let new_y = y as i32 - delta;
    (new_x, new_y)
}

fn square_center_vector_from_coord(x: f32, y: f32, piece_size: f32, half_piece: f32) -> Vec2 {
    Vec2::new(x * piece_size + half_piece, y * piece_size + half_piece)
}

fn translate_center_coord_vec_to_bottom_left_vec(v: Vec2, window: &Window) -> Vec2 {
    Vec2::new(v.x + (window.width() / 2.), v.y + (window.height() / 2.))
}

fn spawn_pieces(mut commands: Commands, pieces: Res<ChessPieceSprites>, windows: Res<Windows>) {
    let window = windows.get_primary().unwrap();
    let piece_size = window.height() / 8.;
    let half_piece = piece_size / 2.;

    let piece_to_sprite = |piece: PieceSprite, vs: Vec2| -> SpriteSheetBundle {
        let mut sprite = TextureAtlasSprite::new(piece as usize);
        sprite.custom_size = Some(Vec2::splat(piece_size - 10.));

        SpriteSheetBundle {
            sprite,
            texture_atlas: pieces.0.clone(),
            transform: Transform {
                translation: Vec3::new(vs.x, vs.y, 900.),
                scale: Vec3::new(0.9, 0.9, 1.),
                ..default()
            },
            ..default()
        }
    };

    let board = chess::Board::default();

    for &sq in chess::ALL_SQUARES.iter() {
        let (x, y) = translate_square_to_xy(sq);
        let (x, y) = translate_xy_to_center_coord(x, y);
        let vs = square_center_vector_from_coord(x as f32, y as f32, piece_size, half_piece);
        let color = if (x + y + 1) % 2 == 0 {
            BLACK_SQUARE_COLOR
        } else {
            WHITE_SQUARE_COLOR
        };
        commands
            .spawn()
            .insert(Name::new(format!("Square {}", sq)))
            .insert(SquareComponent {
                chess_sq: sq,
                position: Vec2::new(vs.x, vs.y),
                bottom_left_coord: translate_center_coord_vec_to_bottom_left_vec(vs, window),
                piece_size,
            })
            .insert_bundle(SpriteBundle {
                sprite: Sprite { color, ..default() },
                transform: Transform {
                    translation: Vec3::new(vs.x, vs.y, 1.0),
                    scale: Vec3::new(piece_size, piece_size, 1.0),
                    ..default()
                },
                ..default()
            });

        let pc = board.piece_on(sq);
        if pc.is_none() {
            continue;
        }
        let pc = pc.unwrap();
        let color = board.color_on(sq).unwrap();

        commands
            .spawn()
            .insert(Name::new(pc.to_string(color)))
            .insert(PieceComponent { position: vs })
            .insert_bundle(piece_to_sprite(PieceSprite::from_chess(pc, color), vs));
    }

    commands
        .spawn()
        .insert(Name::new("SelectingSquares"))
        .insert(SelectingSquares {
            start: None,
            end: None,
        });

    let spawn_selecting_square = |color: Color| SpriteBundle {
        visibility: Visibility { is_visible: false },
        sprite: Sprite { color, ..default() },
        transform: Transform {
            scale: Vec3::new(piece_size, piece_size, 1.0),
            ..default()
        },
        ..default()
    };

    commands
        .spawn()
        .insert(SelectingStartSquare)
        .insert_bundle(spawn_selecting_square(START_COLOR));

    commands
        .spawn()
        .insert(SelectingEndSquare)
        .insert_bundle(spawn_selecting_square(END_COLOR));

    commands.spawn().insert(BoardComponent { board });
}

fn mouse_select_system(
    board_q: Query<&mut BoardComponent>,
    mut mousebtn_evr: EventReader<MouseButtonInput>,
    windows: Res<Windows>,
    square_query: Query<&SquareComponent>,
    mut selected_query: Query<&mut SelectingSquares>,
) {
    use bevy::input::ButtonState;
    let window = windows.get_primary().unwrap();
    let position = window.cursor_position();
    if position.is_none() {
        return;
    }

    let board = board_q.single();

    for ev in mousebtn_evr.iter() {
        match ev.state {
            ButtonState::Pressed => {
                let position = position.unwrap();
                let mut piece_size = 0f32;
                let found_selected = square_query
                    .iter()
                    .find(|&sq| {
                        piece_size = sq.piece_size;
                        let half_piece = sq.piece_size / 2.;
                        let (bl_x, bl_y) = (
                            sq.bottom_left_coord.x - half_piece,
                            sq.bottom_left_coord.y - half_piece,
                        );
                        let (tr_x, tr_y) = (bl_x + sq.piece_size, bl_y + sq.piece_size);
                        let (pos_x, pos_y) = (position.x, position.y);
                        bl_x < pos_x && bl_y < pos_y && tr_x > pos_x && tr_y > pos_y
                    })
                    .map(|s| s.clone());
                if found_selected.is_none() {
                    continue;
                }
                let found_selected = found_selected.unwrap();
                let mut selected = selected_query.single_mut();
                match (selected.start.as_ref(), selected.end.as_ref()) {
                    (None, None) => {
                        if board.board.piece_on(found_selected.chess_sq).is_some() {
                            selected.start = Some(found_selected);
                        }
                    }

                    // Imposible case
                    (None, Some(_)) => {}

                    (Some(_), None) => {
                        selected.end = Some(found_selected);
                    }

                    // Imposible case, handle_chess_move will set this to (None, None)
                    (Some(_), Some(_)) => {}
                }
            }
            ButtonState::Released => {
                let selected = selected_query.single();
                println!(
                    "Mouse button release: start {:?} end {:?}",
                    selected.start.as_ref().map(|s| s.chess_sq),
                    selected.end.as_ref().map(|s| s.chess_sq)
                );
            }
        }
    }
}

fn highlight_selected(
    selected_q: Query<&SelectingSquares>,
    mut set: ParamSet<(
        Query<(&mut Visibility, &mut Transform), With<SelectingStartSquare>>,
        Query<(&mut Visibility, &mut Transform), With<SelectingEndSquare>>,
    )>,
) {
    selected_q.iter().for_each(|selected| {
        let handle_selected = |component: &Option<SquareComponent>,
                               mut visibility: Mut<Visibility>,
                               mut transform: Mut<Transform>| {
            if let Some(start) = component {
                visibility.is_visible = true;
                let pos = start.position;
                transform.translation = Vec3::new(pos.x, pos.y, 2.);
            } else {
                visibility.is_visible = false;
            }
        };
        for (visibility, transform) in set.p0().iter_mut() {
            handle_selected(&selected.start, visibility, transform);
        }

        for (visibility, transform) in set.p1().iter_mut() {
            handle_selected(&selected.end, visibility, transform);
        }
    })
}

fn handle_chess_move(
    mut commands: Commands,
    mut board_q: Query<&mut BoardComponent>,
    mut selected_q: Query<&mut SelectingSquares>,
    mut piece_q: Query<(Entity, &mut PieceComponent, &mut Transform), With<PieceComponent>>,
) {
    let mut selected = selected_q.single_mut();
    if let (Some(start), Some(end)) = (selected.start.as_ref(), selected.end.as_ref()) {
        let mut board = board_q.single_mut();
        let m = chess::ChessMove::new(start.chess_sq, end.chess_sq, None);
        if board.board.legal(m) {
            board.board = board.board.make_move_new(m);
            for (entity, mut piece, mut transform) in piece_q.iter_mut() {
                if piece.position == end.position {
                    commands.entity(entity).despawn();
                }
                if let Some(en_passant) = board.board.en_passant() {
                    println!("en_passant: {:?}", en_passant);
                }
                if piece.position == start.position {
                    *transform = Transform {
                        translation: Vec3::new(end.position.x, end.position.y, 900.),
                        ..default()
                    };
                    *piece = PieceComponent {
                        position: end.position,
                    };
                }
            }
        }

        // Reset selecting after handled
        selected.start = None;
        selected.end = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::{translate_square_to_xy, translate_xy_to_center_coord};
    use chess::Square;
    use std::iter::zip;

    #[test]
    fn translate_coord_works() {
        let result = translate_xy_to_center_coord(3, 4);
        assert_eq!(result, (-1, 0));
    }

    #[test]
    fn translate_square_works() {
        let results = vec![
            translate_square_to_xy(Square::A1),
            translate_square_to_xy(Square::A8),
            translate_square_to_xy(Square::H1),
            translate_square_to_xy(Square::H8),
            translate_square_to_xy(Square::E4),
        ];
        let expects = vec![(0, 0), (0, 7), (7, 0), (7, 7), (4, 3)];
        for (result, expect) in
            zip(results, expects).collect::<Vec<((usize, usize), (usize, usize))>>()
        {
            assert_eq!(result, expect);
        }
    }
}
