# 🏗️ Titanium Next-Gen: MVC Architecture & Easy Database Management Roadmap

> **Design Specification & Architectural Blueprint for Titanium v7.0.0**

---

## 🎯 1. Executive Summary

While Titanium v6.0.0 popularized **Single-File SSR Components** (inspired by modern Vue/Svelte/PHP SFCs), larger enterprise web applications require a structured **Model-View-Controller (MVC)** design pattern, declarative **ActiveRecord ORM**, and **Visual Database Management**.

Titanium v7.0.0 will introduce a **dual-paradigm engine**:
1. **Single-File Components** (`pages/*.titanium`) for lightweight, high-velocity pages.
2. **Structured MVC Architecture** (`app/models/`, `app/controllers/`, `app/views/`) for complex, domain-driven enterprise applications.

---

## 🏛️ 2. MVC Directory Structure

```
my-titanium-app/
├── titanium.toml                # Core project & database config
├── app/
│   ├── controllers/             # Request handling, validation & business logic
│   │   ├── HomeController.ti
│   │   ├── ProductController.ti
│   │   └── Admin/
│   │       └── OrderController.ti
│   ├── models/                  # ActiveRecord Data Models & Business Rules
│   │   ├── Product.ti
│   │   ├── Order.ti
│   │   └── User.ti
│   └── views/                   # HTML / Jinja Template Partials & Layouts
│       ├── layouts/
│       │   ├── app.html
│       │   └── admin.html
│       ├── products/
│       │   ├── index.html
│       │   └── show.html
│       └── orders/
│           └── success.html
├── config/
│   └── routes.ti                # Explicit routing map
├── db/
│   ├── schema.sql               # Current state of SQLite database
│   └── migrations/              # Timestamped migration files
│       └── 20260907_create_products.sql
└── public/
    └── app.css
```

---

## 🗄️ 3. Titanium ActiveRecord ORM & Fluent Query Builder

Instead of writing raw SQL strings, developers can use a high-level, type-safe **ActiveRecord ORM**:

### 3.1 Model Definition (`app/models/Product.ti`)
```rhai
// Model Definition
let Product = Model.define("products", #{
    primary_key: "id",
    timestamps: true,
    rules: #{
        name: "required|min:3",
        price: "required|numeric",
        stock: "integer"
    },
    // Relationships
    relations: #{
        order_items: HasMany("order_items", "product_id"),
        category: BelongsTo("categories", "category_id")
    }
});
```

### 3.2 Fluent Query Ergonomics in Controllers
```rhai
// 1. Fetch all with sorting & pagination
let featured_products = Product.where("is_featured", 1)
                               .where("stock", ">", 0)
                               .order_by("created_at", "DESC")
                               .limit(10)
                               .get();

// 2. Find by Primary Key
let product = Product.find(params.id);

// 3. Create Record
let new_product = Product.create(#{
    name: body.name,
    price: to_float(body.price),
    stock: to_int(body.stock),
    category: body.category
});

// 4. Update Record
product.update(#{
    price: 299.00,
    stock: product.stock - 1
});

// 5. Delete Record
product.delete();
```

---

## 🎛️ 4. Controller Layer (`app/controllers/ProductController.ti`)

```rhai
// ProductController.ti
export fn index(req, session) {
    let category = req.query.category || "All";
    let products = if category == "All" {
        Product.all()
    } else {
        Product.where("category", category).get()
    };

    return view("products/index", #{
        products: products,
        category: category,
        cart_count: Cart.count_for(session.id())
    });
}

export fn show(req, session) {
    let product = Product.find(req.params.id);
    if product == () {
        return not_found("Product does not exist");
    }

    let related = Product.where("category", product.category)
                         .where("id", "!=", product.id)
                         .limit(3)
                         .get();

    return view("products/show", #{
        product: product,
        related: related
    });
}
```

---

## 🛠️ 5. Easy Database Management & Web Studio GUI

### 5.1 Interactive Web Studio Database GUI (`http://127.0.0.1:8080/__titanium_studio`)
- **Visual Table Browser:** Search, filter, edit, and delete rows in real-time with inline spreadsheet-style cells.
- **Visual Schema Builder:** Create tables, add columns, foreign keys, and indexes visually without writing raw SQL.
- **Interactive SQL Runner:** Instant query execution with JSON/CSV export.
- **Migration Manager:** View applied migrations, pending migrations, and rollbacks with 1-click execution.

### 5.2 CLI Code Generators
```bash
# Generate complete MVC scaffold
titanium make:scaffold Product name:string price:float stock:integer

# Generate standalone Model & Migration
titanium make:model User
titanium make:migration add_avatar_to_users

# Run database migrations
titanium db:migrate
titanium db:seed
titanium db:rollback
```

---

## 📅 6. Release Milestones for v7.0.0

| Milestone | Target Feature | Status |
| :--- | :--- | :--- |
| **v7.0.0-alpha** | ActiveRecord ORM runtime (`Model.define`, `Model.find`, `Model.where`) | 📝 Planned |
| **v7.0.0-beta** | MVC Controller dispatcher & view resolution engine | 📝 Planned |
| **v7.0.0-rc1** | Visual Studio GUI Database table manager & schema designer | 📝 Planned |
| **v7.0.0-final** | CLI Scaffolding (`titanium make:model`, `titanium make:controller`) | 🚀 Ready for Development |
