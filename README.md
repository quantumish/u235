![](https://i.imgur.com/vIqhmG7.png)

<div align="center" style="text-align:center;">
	You've heard of atomic integers. Now, get ready for <b>nuclear</b> integers.
</div>

<br>
<code>u235</code> provides a 235-bit unsigned integer type with some special properties. Every 703,800,000 nanoseconds, the value of the integer halves. 
On top of that, it's <i>radioactive</i>, flipping bits nearby it on the stack. Naturally, it can only be used in <code>unsafe</code> blocks. <br><br>

Here's an example: 
```rust
unsafe {
    let x = u235::new(16);
    assert!(x.to_u64() > 0);
    thread::sleep(Duration::from_secs(2));
    assert_eq!(x.to_u64(), 0);
}
```

Is there any defense against the ruinous bit flips `u235`s cause? Never fear! Use a Hazmat to safely(ish) interact with it: take your pick of 
the `OkHazmat`, `GoodHazmat`, and `GreatHazmat` — providing 1,2, and 3 standard deviations worth of padding around your `u235` respectively.
```rust
let mut haz: OkHazmat<u235> = HazmatManufacturer::ok_hazmat();
haz.contain(u235::new(10));
```

Want to live even more dangerously? Enable the experimental `ambient-radiation` feature to have `u235` integers flip bits around them even 
when they're not being used. How? By spawning their own thread and manipulating the main thread's stack from there. Is this UB? Yes. Very much so.
Not just UB in the "you shouldn't technically do this" sense but in the "this has an 80% chance of crashing your program" sense. Use at your own risk
(generally seems to work in release mode).
