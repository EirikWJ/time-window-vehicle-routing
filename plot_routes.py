import json
import matplotlib.pyplot as plt

with open("individual.json", "r", encoding="utf-8") as f:
    data = json.load(f)

depot = data["depot"]
patients = data["patients"]   
routes = data["routes"]

fig, ax = plt.subplots(figsize=(10, 7))
ax.set_aspect("equal", adjustable="box")
ax.set_xticks([])
ax.set_yticks([])
for spine in ax.spines.values():
    spine.set_visible(False)

# Colormap for distinct route colors
cmap = plt.get_cmap("tab10")  

# Plot routes first 
for i, route in enumerate(routes):
    if not route:
        continue

    # Build path: depot to route to depot
    xs = [depot["x"]]
    ys = [depot["y"]]
    for pid in route:
        p = patients[str(pid)]
        xs.append(p["x"])
        ys.append(p["y"])
    xs.append(depot["x"])
    ys.append(depot["y"])

    ax.plot(
        xs, ys,
        linewidth=1.6,
        alpha=0.65,
        color=cmap(i % 10),
        zorder=1
    )

# Plot patients 
px = [patients[k]["x"] for k in patients]
py = [patients[k]["y"] for k in patients]
ax.scatter(px, py, s=18, marker="s", color="black", zorder=3)

# Plot depot 
ax.scatter([depot["x"]], [depot["y"]], s=380, marker="o", color="black", zorder=4)

plt.tight_layout()
plt.show()
