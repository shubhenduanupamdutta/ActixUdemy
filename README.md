# Code Along for the Rust Actix course on Udemy [Build API Servers with Rust and Actix](https://www.udemy.com/course/learn-rust-actix-web-and-sqlx/)

## This is a demo of api clone of twitter

---

## Extractor

### What is an extractor?

The parameters which are going to be used in our actix app, like getting post body, globally shared
app state, which is all being inserted via parameters, are basically extract.
Furthermore, although the data is in string format or json body, properly set extractors make sure
that they are transformed into statically typed rust types.

## NOTE ON `HttpServer` Setup

**When we setup our server, although we set only one time, but there is going to be multiple instance set, default behavior is going to be one instance per core. If we want truly global state across all the individual threads, then we actually instantiate the state data, and then we need to clone it. Behind the scene, actix automatically wraps the data into an `Arc`. Cloning makes sure that each of the thread are the owner of real data.**
