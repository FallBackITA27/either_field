# either_field
`either_field` is a Crate which allows you to create variants of structs where fields have different types

## Why would you use this
While normally you can do a similar thing with enums and their variants, nothing stops you from changing a field's enum variant.

Otherwise, using traits, there is a lot of verbosity, which leads to several dozens lines of codes for just a few generic types in a struct.

This crate instead allows you to generate separate types from a (under the hood) generic struct.