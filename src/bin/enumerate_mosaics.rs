use eternity_ii::rectangular::RectangularRotation;
use eternity_ii::report::{format_ratio, Table};
use eternity_ii::set::builder::counting_sampling::CountingSamplingSetBuilder;
use eternity_ii::set::builder::in_memory_rectangular_mosaic::InMemoryRectangularMosaicSetBuilder;
use eternity_ii::set::builder::in_memory_square_mosaic::InMemorySquareMosaicSetBuilder;
use eternity_ii::set::{
    build_1x1_sets, build_1x1_sets_with_clues, build_rectangles, build_rectangular_centers,
    build_rectangular_corners, build_rectangular_edges, build_square_centers, build_square_corners,
    build_square_edges, build_squares,
};
use eternity_ii::Rotation;

fn main() {
    if false {
        enumerate_mosaics();
    } else {
        enumerate_mosaics_with_hints();
    }
}

fn enumerate_mosaics() {
    let mut table = Table::default();

    let (square_1x1_corners, square_1x1_edges, square_1x1_centers) = build_1x1_sets();
    table.check_square("1x1 corner mosaics", &square_1x1_corners);
    table.check_square("1x1 edge mosaics", &square_1x1_edges);
    table.check_square("1x1 center mosaics", &square_1x1_centers);
    table.print();

    let rectangular_2x1_corners = table.track_build_rectangle("2x1 corner mosaics", || {
        build_rectangular_corners(
            InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
            &square_1x1_corners,
            &square_1x1_edges,
        )
    });
    let rectangular_2x1_edges = table.track_build_rectangle("2x1 edge mosaics", || {
        build_rectangular_edges(
            InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
            &square_1x1_edges,
            &square_1x1_centers,
        )
    });
    let rectangular_2x1_centers = table.track_build_rectangle("2x1 center mosaics", || {
        build_rectangular_centers(
            InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
            &square_1x1_centers,
        )
    });
    table.print();

    let square_2x2_corners = table.track_build_square("2x2 corner mosaics", || {
        build_square_corners(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_corners,
            &rectangular_2x1_edges,
        )
    });
    let square_2x2_edges = table.track_build_square("2x2 edge mosaics", || {
        build_square_edges(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_edges,
        )
    });
    let square_2x2_centers = table.track_build_square("2x2 center mosaics", || {
        build_square_centers(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_centers,
        )
    });
    table.print();

    let _rectangular_4x2_corners = table.track_build_rectangle("4x2 corner mosaics", || {
        build_rectangular_corners(
            InMemoryRectangularMosaicSetBuilder::<4, 2, _>::new(),
            &square_2x2_corners,
            &square_2x2_edges,
        )
    });
    // This is where building in memory starts going into swap on my machine.
    // Switch to counting for the 4x2 edge and center sets.
    table.track_count_and_sample("4x2 edge mosaics", || {
        build_rectangular_edges::<2, 4, _, _, _>(
            CountingSamplingSetBuilder::new(),
            &square_2x2_edges,
            &square_2x2_centers,
        )
    });
    table.track_count_and_sample("4x2 center mosaics", || {
        build_rectangular_centers::<2, 4, _, _>(
            CountingSamplingSetBuilder::new(),
            &square_2x2_centers,
        )
    });
    table.print();
}

fn enumerate_mosaics_with_hints() {
    let mut table = Table::default();

    let (
        square_1x1_corners_no_clues,
        square_1x1_edges_no_clues,
        square_1x1_centers_no_clues,
        square_1x1_centers_c3_clue,
        square_1x1_centers_c14_clue,
        square_1x1_centers_i8_clue,
        square_1x1_centers_n3_clue,
        square_1x1_centers_n14_clue,
    ) = build_1x1_sets_with_clues();
    table.check_square("1x1 corners, no clues", &square_1x1_corners_no_clues);
    table.check_square("1x1 edges, no clues", &square_1x1_edges_no_clues);
    table.check_square("1x1 centers, no clues", &square_1x1_centers_no_clues);
    table.check_square("1x1 centers, C3 clue", &square_1x1_centers_c3_clue);
    table.check_square("1x1 centers, C14 clue", &square_1x1_centers_c14_clue);
    table.check_square("1x1 centers, I8 clue", &square_1x1_centers_i8_clue);
    table.check_square("1x1 centers, N3 clue", &square_1x1_centers_n3_clue);
    table.check_square("1x1 centers, N14 clue", &square_1x1_centers_n14_clue);
    let square_1x1_centers_total = square_1x1_centers_no_clues.len()
        + square_1x1_centers_c3_clue.len()
        + square_1x1_centers_c14_clue.len()
        + square_1x1_centers_i8_clue.len()
        + square_1x1_centers_n3_clue.len()
        + square_1x1_centers_n14_clue.len();
    table.push(
        "1x1 centers, TOTAL",
        square_1x1_centers_total,
        format_ratio(square_1x1_centers_no_clues.len(), square_1x1_centers_total),
    );
    table.print();

    let rectangular_2x1_corners_no_clues =
        table.track_build_rectangle("2x1 corners, no clues", || {
            build_rectangular_corners(
                InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
                &square_1x1_corners_no_clues,
                &square_1x1_edges_no_clues,
            )
        });
    let rectangular_2x1_edges_no_clues = table.track_build_rectangle("2x1 edges, no clues", || {
        build_rectangular_edges(
            InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
            &square_1x1_edges_no_clues,
            &square_1x1_centers_no_clues,
        )
    });
    let rectangular_2x1_centers_no_clues =
        table.track_build_rectangle("2x1 centers, no clues", || {
            build_rectangular_centers(
                InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
                &square_1x1_centers_no_clues,
            )
        });
    let rectangular_2x1_centers_c3_clue =
        table.track_build_rectangle("2x1 centers, C3 clue", || {
            build_rectangles(
                InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
                &square_1x1_centers_c3_clue,
                |a| a.rotation == Rotation::Identity,
                &square_1x1_centers_no_clues,
                |_| true,
            )
        });
    let rectangular_2x1_centers_c14_clue =
        table.track_build_rectangle("2x1 centers, C14 clue", || {
            build_rectangles(
                InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
                &square_1x1_centers_no_clues,
                |_| true,
                &square_1x1_centers_c14_clue,
                |b| b.rotation == Rotation::Identity,
            )
        });
    let rectangular_2x1_centers_i8_clue =
        table.track_build_rectangle("2x1 centers, I8 clue", || {
            build_rectangles(
                InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
                &square_1x1_centers_no_clues,
                |_| true,
                &square_1x1_centers_i8_clue,
                |b| b.rotation == Rotation::Identity,
            )
        });
    let rectangular_2x1_centers_n3_clue =
        table.track_build_rectangle("2x1 centers, N3 clue", || {
            build_rectangles(
                InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
                &square_1x1_centers_n3_clue,
                |a| a.rotation == Rotation::Identity,
                &square_1x1_centers_no_clues,
                |_| true,
            )
        });
    let rectangular_2x1_centers_n14_clue =
        table.track_build_rectangle("2x1 centers, N14 clue", || {
            build_rectangles(
                InMemoryRectangularMosaicSetBuilder::<2, 1, _>::new(),
                &square_1x1_centers_no_clues,
                |_| true,
                &square_1x1_centers_n14_clue,
                |b| b.rotation == Rotation::Identity,
            )
        });
    let rectangular_2x1_centers_total = rectangular_2x1_centers_no_clues.len()
        + rectangular_2x1_centers_c3_clue.len()
        + rectangular_2x1_centers_c14_clue.len()
        + rectangular_2x1_centers_i8_clue.len()
        + rectangular_2x1_centers_n3_clue.len()
        + rectangular_2x1_centers_n14_clue.len();
    table.push(
        "2x1 centers, TOTAL",
        rectangular_2x1_centers_total,
        format_ratio(
            rectangular_2x1_centers_no_clues.len(),
            rectangular_2x1_centers_total,
        ),
    );
    table.print();

    let square_2x2_corners_no_clues = table.track_build_square("2x2 corners, no clues", || {
        build_square_corners(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_corners_no_clues,
            &rectangular_2x1_edges_no_clues,
        )
    });
    let square_2x2_edges_no_clues = table.track_build_square("2x2 edges, no clues", || {
        build_square_edges(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_edges_no_clues,
        )
    });
    let square_2x2_centers_no_clues = table.track_build_square("2x2 centers, no clues", || {
        build_square_centers(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_centers_no_clues,
        )
    });
    let square_2x2_centers_c3_clue = table.track_build_square("2x2 centers, C3 clue", || {
        build_squares(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_centers_c3_clue,
            |a| a.rotation == RectangularRotation::Identity,
            &rectangular_2x1_centers_no_clues,
            |_| true,
        )
    });
    let square_2x2_centers_c14_clue = table.track_build_square("2x2 centers, C14 clue", || {
        build_squares(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_centers_c14_clue,
            |a| a.rotation == RectangularRotation::Identity,
            &rectangular_2x1_centers_no_clues,
            |_| true,
        )
    });
    let square_2x2_centers_i8_clue = table.track_build_square("2x2 centers, I8 clue", || {
        build_squares(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_centers_i8_clue,
            |a| a.rotation == RectangularRotation::Identity,
            &rectangular_2x1_centers_no_clues,
            |_| true,
        )
    });
    let square_2x2_centers_n3_clue = table.track_build_square("2x2 centers, N3 clue", || {
        build_squares(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_centers_no_clues,
            |_| true,
            &rectangular_2x1_centers_n3_clue,
            |b| b.rotation == RectangularRotation::Identity,
        )
    });
    let square_2x2_centers_n14_clue = table.track_build_square("2x2 centers, N14 clue", || {
        build_squares(
            InMemorySquareMosaicSetBuilder::new(),
            &rectangular_2x1_centers_no_clues,
            |_| true,
            &rectangular_2x1_centers_n14_clue,
            |b| b.rotation == RectangularRotation::Identity,
        )
    });
    let square_2x2_centers_total = square_2x2_centers_no_clues.len()
        + square_2x2_centers_c3_clue.len()
        + square_2x2_centers_c14_clue.len()
        + square_2x2_centers_i8_clue.len()
        + square_2x2_centers_n3_clue.len()
        + square_2x2_centers_n14_clue.len();
    table.push(
        "2x2 centers, TOTAL",
        square_2x2_centers_total,
        format_ratio(square_2x2_centers_no_clues.len(), square_2x2_centers_total),
    );
    table.print();

    let _rectangular_4x2_corners_no_clues =
        table.track_build_rectangle("4x2 corners, no clues", || {
            build_rectangular_corners::<2, 4, _, _, _>(
                InMemoryRectangularMosaicSetBuilder::new(),
                &square_2x2_corners_no_clues,
                &square_2x2_edges_no_clues,
            )
        });
    let rectangular_4x2_edges_no_clues =
        table.track_count_and_sample("4x2 edges, no clues", || {
            build_rectangular_edges::<2, 4, _, _, _>(
                CountingSamplingSetBuilder::new(),
                &square_2x2_edges_no_clues,
                &square_2x2_centers_no_clues,
            )
        });
    let rectangular_4x2_edges_c3_clue = table.track_count_and_sample("4x2 edges, C3 clue", || {
        build_rectangles::<2, 4, _, _, _>(
            CountingSamplingSetBuilder::new(),
            &square_2x2_edges_no_clues,
            |a| a.rotation == Rotation::Identity,
            &square_2x2_centers_c3_clue,
            |b| b.rotation == Rotation::Identity,
        )
    });
    let rectangular_4x2_edges_c14_clue =
        table.track_count_and_sample("4x2 edges, C14 clue", || {
            build_rectangles::<2, 4, _, _, _>(
                CountingSamplingSetBuilder::new(),
                &square_2x2_centers_c14_clue,
                |a| a.rotation == Rotation::Identity,
                &square_2x2_edges_no_clues,
                |b| b.rotation == Rotation::HalfTurn,
            )
        });
    let rectangular_4x2_edges_n3_clue = table.track_count_and_sample("4x2 edges, N3 clue", || {
        build_rectangles::<2, 4, _, _, _>(
            CountingSamplingSetBuilder::new(),
            &square_2x2_edges_no_clues,
            |a| a.rotation == Rotation::Identity,
            &square_2x2_centers_n3_clue,
            |b| b.rotation == Rotation::Identity,
        )
    });
    let rectangular_4x2_edges_n14_clue =
        table.track_count_and_sample("4x2 edges, N14 clue", || {
            build_rectangles::<2, 4, _, _, _>(
                CountingSamplingSetBuilder::new(),
                &square_2x2_centers_n14_clue,
                |a| a.rotation == Rotation::Identity,
                &square_2x2_edges_no_clues,
                |b| b.rotation == Rotation::HalfTurn,
            )
        });
    let rectangular_4x2_edges_total = rectangular_4x2_edges_no_clues
        + rectangular_4x2_edges_c3_clue
        + rectangular_4x2_edges_c14_clue
        + rectangular_4x2_edges_n3_clue
        + rectangular_4x2_edges_n14_clue;
    table.push(
        "4x2 edges, TOTAL",
        rectangular_4x2_edges_total,
        format_ratio(rectangular_4x2_edges_no_clues, rectangular_4x2_edges_total),
    );
    // 20,382,606,825 elements
    // Takes 163.016 s to count
    let rectangular_4x2_centers_no_clues =
        table.track_count_and_sample("4x2 centers, no clues", || {
            build_rectangular_centers::<2, 4, _, _>(
                CountingSamplingSetBuilder::new(),
                &square_2x2_centers_no_clues,
            )
        });
    let rectangular_4x2_centers_i8_clue =
        table.track_count_and_sample("4x2 centers, I8 clue", || {
            build_rectangles::<2, 4, _, _, _>(
                CountingSamplingSetBuilder::new(),
                &square_2x2_centers_no_clues,
                |_| true,
                &square_2x2_centers_i8_clue,
                |b| b.rotation == Rotation::Identity,
            )
        });
    let rectangular_4x2_centers_total =
        rectangular_4x2_centers_no_clues + rectangular_4x2_centers_i8_clue;
    table.push(
        "4x2 centers, TOTAL",
        rectangular_4x2_centers_total,
        format_ratio(
            rectangular_4x2_centers_no_clues,
            rectangular_4x2_centers_total,
        ),
    );
    table.print();
}
