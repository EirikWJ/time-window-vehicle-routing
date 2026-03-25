$ErrorActionPreference = "Stop"

cargo run #--release

python plot_routes.py

python plot_fitness.py