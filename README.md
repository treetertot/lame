# lame
A simple 2d game engine written in rust, but multithreaded

I made a basic game engine using the pixel go library, had collision detection, entities, etc. Unfortunately, I hadn't yet figured out git, so when I installed a new distro WITHOUT BACKING UP, it all got wiped.
I made this repository for a rewrite, but never got around to it. UNTIL NOW!

Since the sad loss of lame I have learned the rust programming language and found it much better suited to the task.
That being said the go threadpool was pretty neat and let me simulate an absurd number of bouncing boxes.
Due to the way I handled entities in go being horribly not thread safe, I can't replicate that lame, but due to crossbeam and rust's incredible concurrency I can still use a threadpool.
