# Vehicle routing problem with time windows


### Repository Structure
```text
./
 ├── datasets/
 ├── results/
 │    ├── crowding/
 │    └── mu_lambda/
 └── src/
      └── main.rs
```
### Dataset Structure
```javascript
{
    "instance_name": "train_0",
    "nbr_nurses": 25,
    "capacity_nurse": 200,
    "benchmark": 827.3,
    "depot": {
        "return_time": 1236,
        "x_coord": 40,
        "y_coord": 50
    },
    "patients": {
        "1": {
            "x_coord": 45,
            "y_coord": 68,
            "demand": 10,
            "start_time": 0,
            "end_time": 1217,
            "care_time": 90
        },

        ...

        "N": {
            "x_coord": 42,
            "y_coord": 66,
            "demand": 10,
            "start_time": 0,
            "end_time": 1219,
            "care_time": 90
        },
    },
    "travel_times": [
        [ 0.0, 18.6, 20.6, 16.1, ... , 18.1, 15.1, 19.0, 16.0 ],
        ...
        [ 43.0, 40.6, 37.2, 36.0, ... , 40.3, 30.8, 33.5, 38.0 ]
    ]
}
```