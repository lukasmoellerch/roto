# Roto

Grpc has .proto files, Thrift has .thrift, but what do standard REST APIs have? Nothing. Ok well, they have OpenAPI, but that's barely human readable and definitely not something you'd want to write by hand. Roto tries to tackle that problem. It's designed to be human readable and writable, and to be easy to parse and generate code from for various programming languages. Although it is designed with REST APIs in mind, it is mainly a language for programming with types, and can be used for other things as well.

At the moment it's mostly just a language with a couple of examples, a crappy parser, and code generation for Python. We'll see how it goes.

The focus at the moment is on the type system which is a bit of a mixture between rust and typescript - treating types as values while having a rust-like syntax with variants and structs being the main building blocks. Here's an example of what the type system looks like:

```rust
type Identified<T> = T & struct {
  id: string,
};


type CollectionResponse<T> = struct {
  items: T,
  total: int,
  cursor: string,
};

type UserStatus = enum {
  active(unit),
  inactive(unit),
  pending(unit),
};

type UserProperties = struct {
    name: string,
    age: int,
    status: UserStatus,
};


type User = Identified<T=UserProperties>;
type UserCollectionResponse = CollectionResponse<T=User>;

type X = UserCollectionResponse;
```

This high-level type code is compiled to a more simple intermediate representation (roto-ir) that removes generics and operators. Here's the intermediate representation for the above code:

```rust
type X#0 = reference 1
type UserCollectionResponse#1 = reference 2
type CollectionResponse<T=Variable("User")>#2 = struct {
  total: int,
  cursor: string,
  items: reference 3,
}
type User#3 = reference 4
type Identified<T=Variable("UserProperties")>#4 = struct {
  name: string,
  age: int,
  status: reference 6,
  id: string,
}
type UserProperties#5 = struct {
  name: string,
  age: int,
  status: reference 6,
}
type UserStatus#6 = enum {
  inactive(unit),
  pending(unit),
  active(unit),
}
```

Roto-ir is in-memory at the moment, the example above is just a simple textual representation. The idea is that this intermediate representation can be used to generate code for various programming languages.

The parser and the frontend for this works and handles intersection types, generics and self-recursive types. The backend is not implemented yet.

Will this ever be a thing? Who knows.