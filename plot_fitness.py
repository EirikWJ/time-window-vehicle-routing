import argparse
import csv
import math
from collections import defaultdict
from pathlib import Path

import matplotlib.pyplot as plt


def load_fitness_history(
    csv_path: Path,
) -> dict[int, list[tuple[int, float, float, float, float]]]:
    runs: dict[int, list[tuple[int, float, float, float, float]]] = defaultdict(list)

    with csv_path.open("r", newline="", encoding="utf-8") as file:
        reader = csv.DictReader(file)
        required = {
            "run",
            "generation",
            "min_fitness",
            "mean_fitness",
            "max_fitness",
            "entropy",
        }
        missing = required.difference(reader.fieldnames or [])
        if missing:
            raise ValueError(f"CSV is missing columns: {sorted(missing)}")

        for row in reader:
            run = int(row["run"])
            generation = int(row["generation"])
            min_fit = float(row["min_fitness"])
            mean_fit = float(row["mean_fitness"])
            max_fit = float(row["max_fitness"])
            entropy = float(row["entropy"])
            runs[run].append((generation, min_fit, mean_fit, max_fit, entropy))

    for run_rows in runs.values():
        run_rows.sort(key=lambda item: item[0])

    return runs


def clip_above_threshold(
    generations: list[int], values: list[float], threshold: float
) -> tuple[list[float], list[int]]:
    clipped: list[float] = []
    overflow_generations: list[int] = []

    for generation, value in zip(generations, values):
        if value > threshold:
            clipped.append(math.nan)
            overflow_generations.append(generation)
        else:
            clipped.append(value)

    return clipped, overflow_generations


def draw_overflow_connectors(
    ax: plt.Axes,
    generations: list[int],
    values: list[float],
    threshold: float,
    color: str,
) -> None:
    for i, value in enumerate(values):
        if value <= threshold:
            continue

        x_curr = generations[i]

        if i > 0 and values[i - 1] <= threshold:
            ax.plot(
                [generations[i - 1], x_curr],
                [values[i - 1], threshold],
                linestyle="--",
                linewidth=1.1,
                color=color,
                alpha=0.8,
            )

        if i + 1 < len(values) and values[i + 1] <= threshold:
            ax.plot(
                [x_curr, generations[i + 1]],
                [threshold, values[i + 1]],
                linestyle="--",
                linewidth=1.1,
                color=color,
                alpha=0.8,
            )


def plot_run(
    run: int,
    rows: list[tuple[int, float, float, float, float]],
    fitness_max: float,
) -> None:
    generations = [row[0] for row in rows]
    min_vals = [row[1] for row in rows]
    mean_vals = [row[2] for row in rows]
    max_vals = [row[3] for row in rows]
    entropy_vals = [row[4] for row in rows]

    fig, (ax_fit, ax_ent) = plt.subplots(2, 1, figsize=(10, 8), sharex=True)

    series = [
        ("Min fitness", min_vals, "tab:blue"),
        ("Mean fitness", mean_vals, "tab:orange"),
        ("Max fitness", max_vals, "tab:green"),
    ]

    marker_label_added = False
    for label, values, color in series:
        clipped, overflow_generations = clip_above_threshold(generations, values, fitness_max)

        ax_fit.plot(generations, clipped, label=label, linewidth=2.0, color=color)
        draw_overflow_connectors(ax_fit, generations, values, fitness_max, color)

        if overflow_generations:
            ax_fit.scatter(
                overflow_generations,
                [fitness_max] * len(overflow_generations),
                marker="^",
                color=color,
                s=26,
                label=(f"> {fitness_max:g} (out of view)" if not marker_label_added else None),
                zorder=3,
            )
            marker_label_added = True

    ax_fit.set_title(f"Fitness and entropy over epochs (run {run})")
    ax_fit.set_ylabel("Fitness")
    ax_fit.set_ylim(top=fitness_max)
    ax_fit.grid(True, alpha=0.25)
    ax_fit.legend()

    ax_ent.plot(
        generations,
        entropy_vals,
        label="Route edge entropy",
        color="black",
        linewidth=2.0,
    )
    ax_ent.set_xlabel("Generation")
    ax_ent.set_ylabel("Route Edge Entropy")
    ax_ent.set_ylim(0.0, 1.05)
    ax_ent.grid(True, alpha=0.25)
    ax_ent.legend()

    fig.tight_layout()


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Plot min/mean/max fitness and entropy from GA CSV history"
    )
    parser.add_argument(
        "--csv",
        type=Path,
        default=Path("fitness_history.csv"),
        help="Path to fitness history CSV",
    )
    parser.add_argument(
        "--run",
        type=int,
        default=None,
        help="Run number to plot (default: latest run)",
    )
    parser.add_argument(
        "--fitness-max",
        type=float,
        default=2500.0,
        help="Upper y-limit for fitness. Values above are hidden with indicators.",
    )
    args = parser.parse_args()

    if not args.csv.exists():
        raise FileNotFoundError(f"CSV file not found: {args.csv}")

    runs = load_fitness_history(args.csv)
    if not runs:
        raise ValueError("CSV contains no rows")

    selected_run = args.run if args.run is not None else max(runs)
    if selected_run not in runs:
        raise ValueError(f"Run {selected_run} not found. Available runs: {sorted(runs)}")

    plot_run(selected_run, runs[selected_run], args.fitness_max)
    plt.show()


if __name__ == "__main__":
    main()
