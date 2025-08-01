# FrostyEngine
A simple game engine made in rust

## Midsummer Update
Direct engine development has been temporarily halted. I initially started working on FrostyEngine because I noticed a lot of my graphics code would be reused from project to project, so I wanted to have a simple library I could import. Coding then became much more about understanding thread safety and fiddling with bits rather than actually moving towards a game. Design decisions (i.e. the current rendering set up) were being made in vacuums with tailored examples rather than in naturally in the wild.

All of this is to say, I've been spending time making a game. Progress has been rather slow, but it's really helped me see the shortcomings of the current engine. I've also been able to work on systems that otherwise wouldn't be further down the roadmap - such as physics. All in all engine development hasn't stopped as much as it's entered a more hidden RnD phase. As things are developed and finalized I'll add them, but for the time being there won't be much activity in this repo.

## TODO
### Alloc
- Write tests for all structs
- Ensure safety of unsafe blocks
### Render
- Write tests for all structs
### Core
- Write tests for Entity
- Implement sibling references for Entity components
#### Concur
- Make real constructors for Lockless Queues
- Implement reference counting for inner vec
- Properly clean up as pushers and receivers are dropped
