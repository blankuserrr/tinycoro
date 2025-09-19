//! Basic example demonstrating how to use tinycoro coroutines
//!
//! This example shows:
//! - Creating a coroutine with a simple function
//! - Resuming and yielding coroutines
//! - Pushing and popping data to/from coroutine storage
//! - Checking coroutine status

use tinycoro::Coroutine;

/// A simple coroutine function that yields twice and uses storage
unsafe extern "C" fn example_coroutine(_co: *mut tinycoro::mco_coro) {
    println!("Coroutine started!");

    // Now we can use the safe yield_current function
    println!("About to yield for the first time...");
    let _ = tinycoro::yield_current();
    println!("Resumed after first yield!");

    // Yield again
    println!("About to yield for the second time...");
    let _ = tinycoro::yield_current();
    println!("Resumed after second yield!");

    println!("Coroutine finished!");
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Tinycoro Basic Example ===\n");

    // Create a new coroutine with 64KB stack
    let mut coroutine = unsafe { Coroutine::new(example_coroutine, 64 * 1024)? };

    println!("Created coroutine with status: {:?}", coroutine.status());

    // Push some data to the coroutine storage
    let data = 42u32;
    coroutine.push(&data)?;
    println!("Pushed data: {}", data);
    println!("Bytes stored: {}", coroutine.bytes_stored());
    println!("Storage size: {}", coroutine.storage_size());

    // Resume the coroutine (first time)
    println!("\n--- First Resume ---");
    coroutine.resume()?;
    println!(
        "Coroutine status after first resume: {:?}",
        coroutine.status()
    );

    // Resume the coroutine (second time)
    println!("\n--- Second Resume ---");
    coroutine.resume()?;
    println!(
        "Coroutine status after second resume: {:?}",
        coroutine.status()
    );

    // Resume the coroutine (final time)
    println!("\n--- Final Resume ---");
    coroutine.resume()?;
    println!(
        "Coroutine status after final resume: {:?}",
        coroutine.status()
    );

    // Pop the data back from storage
    let retrieved_data: u32 = coroutine.pop()?;
    println!("\nPopped data: {}", retrieved_data);
    assert_eq!(data, retrieved_data);

    println!("Bytes stored after pop: {}", coroutine.bytes_stored());

    // Demonstrate different data types
    println!("\n--- Working with different data types ---");

    // Push a string
    let text = "Hello, coroutines!";
    coroutine.push(&text)?;
    println!("Pushed string: '{}'", text);

    // Push a struct
    #[derive(Debug, PartialEq)]
    struct Point {
        x: i32,
        y: i32,
    }
    let point = Point { x: 10, y: 20 };
    coroutine.push(&point)?;
    println!("Pushed point: {:?}", point);

    println!("Total bytes stored: {}", coroutine.bytes_stored());

    // Pop them back (LIFO order)
    let retrieved_point: Point = coroutine.pop()?;
    let retrieved_text: &str = coroutine.pop()?;

    println!("Popped point: {:?}", retrieved_point);
    println!("Popped string: '{}'", retrieved_text);

    assert_eq!(point, retrieved_point);
    assert_eq!(text, retrieved_text);

    println!("\n=== Example completed successfully! ===");

    Ok(())
}
