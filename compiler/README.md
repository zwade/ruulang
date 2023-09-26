# The RuuLang DSL

This package provides a parser for the `ruulang` DSL.

`ruulang` is a language used to describe security configurations. Specifically, it can be used to define succinct access-based policies in an arbitrary security domain. Unlike other approaches, ruulang describes policies based on the edges used to get from one known state to another, and then describes the attributes that are acquired along the way.

## Example

Consider a simple example where there are 3 entities

- Users
- Companies
- Secrets

In addition, there are a number of edges between these entities

- `employee` maps a user to a company -- with an optional `role` attribute
- `customer` also maps a user to a company
- `owner` maps a company to another company
- `affiliate` likewise maps a company to another company
- `secret` maps a company to a secret (eponymous with the entity `Secret` because it is a unique access pattern)

One might right the following ruulang policy

```ruulang
@User {
    employee:role(basic) {
        read;

        owner { read; }
        affiliate { read-basic; }
    }

    employee:role(admin) {
        write;

        secret { read; }

        owner {
            read;
            write;
        }
    }

    customer {
        read-basic;
    }
}
```

In this example, employees with the `basic` role can read information about the company, companies it owns, and read basic information about affiliated companies. Admins can do more -- they can write data to the company and companies it owns, as well as read secrets. Customers, in contrast, can only read basic information about the company.

## Syntax

### Entrypoints

```ruulang
@<entity name> {

}

@<another entity name> {

}
```

All top level rules must have an entrypoint. This is denoted by the `@` prefix and specifies an **entity**, not an **edge**. This is the type that one is starting with. Often this will be some user type, but when evaluating rules across multiple tenancies (to be discussed in a future revision), it may be helpful to have other types of entrypoints.

### Rules

```ruulang
@Entity {
    <rule> {

    }
}
```

A rules is a single edge, and all of the data contained within it. All rules are allowed to recursively contain other rules, although none need to.

### Attributes

```ruulang
@Entity {
    rule :<attribute1>(<arg1>, <arg2>) :<attribute2> {

    }
}
```

Attributes are modifiers on rules. They may take an arbitrary number of arguments (including 0), and should be considered something unique to that edge. For instance, `:role(admin)` is a classic example of an attribute that would exist between a user and a company.

### Grants
```ruulang
@Entity {
    rule :attr {
        <grant1>;
        <grant2>;
    }
}
```

Finally, grants are the specific policies that are granted to resulting **entities** after evaluating the policy. These are what will ultimately be checked when determining whether access should be granted, e.g. `read` or `write`.