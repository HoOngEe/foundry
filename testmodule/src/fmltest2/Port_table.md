|      | Host                          | Human                           | Cleric                         | God                             |
| ---- | ----------------------------- | ------------------------------- | ------------------------------ | ------------------------------- |
| 1    | <<**Human**: `WeatherRequest` | >>**Host**: `WeatherRequest`    | <<**God**: `WeatherForecast`   | >>**Cleric**: `WeatherForecast` |
| 2    |                               | <<**Cleric**: `WeatherResponse` | >>**Human**: `WeatherResponse` |                                 |
| 3    | <<**Human**: `PrayRequest`    | >>**Host**: `PrayRequest`       | <<**God**: `RainOracleGiver`   | >>**Cleric**: `RainOracleGiver` |
| 4    |                               | <<**Cleric**: `PrayResponse`    | >>**Human**: `PrayResponse`    |                                 |
| *4'* |                               |                                 | <<***God***: `RainOracle`      | >>***Cleric***: `RainOracle`    |
| 6    |                               | >>**Cleric**: `GroundObserver`  | <<**Human**: `GroundObserver`  |                                 |
| 7    | <<**Human**: `TalkToHumans`   | >>**Host**: `TalkToHumans`      |                                |                                 |
| 8    |                               | <<**Cleric**: `TalkToClerics`   | >>**Human**: `TalkToClerics`   |                                 |
| 9    |                               | >>**Cleric**: `TalkToHumans`    | <<**Human**: `TalkToHumans`    |                                 |
| 10   |                               |                                 | >>**God**:  `TalkToClerics`    | <<**Cleric**: `TalkToClerics`   |
| 11   |                               |                                 | <<**God**: `TalkToGods`        | >>**Cleric**: `TalkToGods`      |
| 12   |                               | >>**God**: `TalkToHumans`       |                                | <<**Human**: `TalkToHumans`     |
| 13   |                               | <<**God**: `TalkToGods`         |                                | >>**Human**: `TalkToGods`       |

