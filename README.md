# either_field
`either_field` is a Crate which allows you to create variants of structs where fields have different types

## Why would you use this
While normally you can do a similar thing with enums and their variants, nothing stops you from changing a field's enum variant.

Otherwise, using traits, there is a lot of verbosity, which leads to several dozens lines of codes for just a few generic types in a struct.

This crate instead allows you to generate separate types from a (under the hood) generic struct.

### E.G:
Picture you're working with SQL, and you have two tables:
- `Scores`

| | player_id | value |
|-|-|-|
|Type| INTEGER | INTEGER |

and
- `Players`

| | player_id | username |
|-|-|-|
|Type| INTEGER | VARCHAR(255) |

Say that in a function, you're returning either the pure `Scores` table, or a modified version with the player
```rs
// Example 1
struct Scores {
    player_id: i32,
    value: i32
}

// Example 2
struct PlayerBasic {
    player_id: PlayerBasic,
    username: String
}

struct ScoresWithPlayer {
    player: PlayerBasic,
    value: i32
}
```

Wouldn't it come in handy if you could define both of these with one struct?
```rs
#[template_table]
struct GenericPlayer {
    player: either!(i32 | PlayerBasic),
    value: i32
}
```

The actual example in the `tests/` folder.