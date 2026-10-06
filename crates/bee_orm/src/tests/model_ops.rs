// Copyright (c) 2026 erik <erik@erik.xyz> — https://erik.xyz
//! SQL-shape checks for the default [`Model`](crate::Model) operations:
//! timestamps, soft delete, hooks, `insert_many`.

use super::*;

#[tokio::test]
async fn insert_without_timestamps_stays_plain() {
    let db = Mock::default();
    assert_eq!(Tiny { id: 1, v: 5 }.insert(&db).await.unwrap(), 1);
    let (sql, params) = db.calls().remove(0);
    assert_eq!(sql, "INSERT INTO tiny_items (v) VALUES (?)");
    assert_eq!(params, vec![Value::Int(5)]);
}

#[tokio::test]
async fn insert_injects_one_now_after_the_own_values() {
    let db = Mock::default();
    note(1).insert(&db).await.unwrap();
    let (sql, params) = db.calls().remove(0);
    assert_eq!(sql, "INSERT INTO notes (title, deleted, created, updated) VALUES (?, ?, ?, ?)");
    assert_eq!(params[0], Value::Text("n".into()));
    assert_eq!(params[1], Value::Bool(false));
    match (&params[2], &params[3]) {
        (Value::Int(created), Value::Int(updated)) => {
            assert!(*created > 0);
            assert_eq!(created, updated);
        }
        other => panic!("expected two timestamps, got {other:?}"),
    }
}

#[tokio::test]
async fn update_refreshes_auto_now_only() {
    let db = Mock::default();
    note(7).update(&db).await.unwrap();
    let (sql, params) = db.calls().remove(0);
    assert_eq!(sql, "UPDATE notes SET title = ?, updated = ? WHERE id = ?");
    assert_eq!(params[0], Value::Text("n".into()));
    assert!(matches!(&params[1], Value::Int(now) if *now > 0));
    assert_eq!(params[2], Value::Int(7));
}

#[tokio::test]
async fn soft_delete_flips_the_flag_and_guards_on_it() {
    let db = Mock::default();
    note(7).delete(&db).await.unwrap();
    let (sql, params) = db.calls().remove(0);
    assert_eq!(sql, "UPDATE notes SET deleted = ? WHERE id = ? AND deleted = ?");
    assert_eq!(params, vec![Value::Bool(true), Value::Int(7), Value::Bool(false)]);
}

#[tokio::test]
async fn hard_delete_is_a_plain_delete() {
    let db = Mock::default();
    note(7).hard_delete(&db).await.unwrap();
    let (sql, params) = db.calls().remove(0);
    assert_eq!(sql, "DELETE FROM notes WHERE id = ?");
    assert_eq!(params, vec![Value::Int(7)]);
}

#[tokio::test]
async fn before_hook_error_aborts_before_any_sql() {
    let db = Mock::default();
    let err = Guarded { id: 1, title: "blocked".into() }.insert(&db).await.unwrap_err();
    assert!(err.to_string().contains("blocked by before_insert"));
    assert!(db.calls().is_empty());
}

#[tokio::test]
async fn after_hook_error_propagates_after_the_write() {
    let db = Mock::default();
    let err = Guarded { id: 1, title: "late".into() }.insert(&db).await.unwrap_err();
    assert!(err.to_string().contains("raised by after_insert"));
    assert_eq!(db.calls().len(), 1);
}

#[tokio::test]
async fn insert_many_chunks_at_the_parameter_ceiling() {
    let db = Mock::default();
    let rows: Vec<Tiny> = (0..1000).map(|v| Tiny { id: v, v }).collect();
    assert_eq!(Tiny::insert_many(&db, &rows).await.unwrap(), 2);
    let calls = db.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].1.len(), 999);
    assert_eq!(calls[1].1.len(), 1);
    assert!(calls[0].0.starts_with("INSERT INTO tiny_items (v) VALUES (?)"));
}

#[tokio::test]
async fn insert_many_of_nothing_does_nothing() {
    let db = Mock::default();
    assert_eq!(Tiny::insert_many(&db, &[]).await.unwrap(), 0);
    assert!(db.calls().is_empty());
}
