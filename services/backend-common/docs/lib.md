# The Backend Common Crate!!!!
> **Note on naming**
>
> The name `backend-common` is mostly historical. It _**used**_ to be only used by servers 
> so at that point it _**was**_ the backend-common (fun fact before, it was simply
> called `common`).
>
> BUTT, As DLN expanded to support multiple client applications, this crate's role kinda changed and grew to
> include functionality shared across both servers and clients. The
> [`dln-core`](../dln_core) crate now provides the higher level public interface
> for applications, while `backend-common` remains the underlying shared
> foundation.

---

This crate exists to exists as a sort of buffer between clients and servers. They get the same information so this crate acts as a single source of truth.

Visually it looks like so:

```text
    [‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾]
    [    Applications w/ dln-core    ]
    [                                ]
    [       CLI / TUI / QtUI         ]
    [________________________________]
                    |
                    ↓
                 dln-core
                    |
                    ↓
              backend-common
                    ↑
                    |
        +-----------+-------------+
        |           |             |
    Auth      GradeGetter   Other Services
```

However, clients should *definitely* not use this crate up front. Clients are advised to use [`dln-core`](../dln_core) instead.

**Servers should be the only package which _directly_ depends on this crate.**
