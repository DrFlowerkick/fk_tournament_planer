---
applyTo: "**"
---

# Domain Knowledge: Tournament System

This document outlines the core concepts, rules, and architecture of the tournament planning application. It serves as the single source of truth for the comprehensive understanding of the domain logic.

## Core Terminology

- **Entrant**: A competitor (individual athlete or team) participating in the tournament.
- **Match**: A direct competition between two entrants. Results can be Win, Loss, or Draw (depending on the sport).
- **Stage**: A distinct phase of the tournament (e.g., "Preliminary Round", "Playoffs"). A tournament consists of one or more stages.
- **Group**: A subset of entrants within a stage. Entrants play against others in their group according to the stage's mode.
- **Round**: A set of matches played simultaneously or within the same time frame defined by the mode (e.g., Round 1 of Swiss System).

## Tournament Structure & Progression

1. **Stages**: Entrants progress through logical stages.
2. **Groups**: Within a stage, entrants are divided into groups.
   - **Initial Seeding**: In the first stage, entrants can be assigned to groups by:
     - **Rank**: Based on external ranking (e.g., World Rank).
     - **Snake Seeding**: Distributing entrants pattern-wise to ensure balanced groups (Standard international practice).
     - **Random**: Random assignment.
   - **Progression Seeding**: For subsequent stages, grouping is determined by the ranking from the previous stage (e.g., Top 4 of Group A go to Group 1 of Stage 2).

## Tournament Modes

### Round Robin / Group Phase

- Standard for early stages.
- Entrants in a group play against every other entrant in that group.
- **Ranking**: Based on accumulated victory points (e.g., Win=1, Draw=0.5).

### Knock Out (KO)

- Typically used in final stages.
- **Mode**: The loser of a match immediately drops out of the tournament.
- **Requirement**: Group size must be a power of 2 ($2^n$).

### KO Play Out

- Similar to KO, but losers continue to play against each other to determine specific lower rankings (e.g., playing for 3rd, 5th place, etc.), rather than just dropping out.

### Swiss System

- Designed for tournaments with many entrants where a full Round Robin is not feasible.
- **Structure**: Modeled as a single stage with one group containing all entrants.
- **Pairing Logic**:
  - **Round 1**: Random or seeded pairing.
  - **Subsequent Rounds**: Entrants are paired with neighbors in the current ranking who they have **not** played yet.
- **Odd Number of Entrants**: One entrant receives a "Bye" (Free Win) per round. An entrant can typically receive only one Bye per tournament.
- **Duration**: Recommended number of rounds is $\log_2(\text{Entrants}) + 2$.
- **Tie Breaker**: Primarily uses **Buchholz score**.

## Ranking & Tie Breaking

Ranking resolves the order of entrants (1 to N) within a group or stage.

1. **Victory Points**: Primary metric (Wins/Draws).
2. **Tie Breakers**: Used when victory points are equal. Can be combined/ordered:
   - **Delta Points**: Score difference (Scored - Conceded).
   - **Total Points**: Total score achieved.
   - **Direct Comparison**: Result of the match between the tied entrants.
   - **Initial Rank**: Fallback to pre-tournament rank.
   - **Buchholz Score**: Sum of opponents' scores (specific to Swiss System).
   - **Coin Flip / Random**: Last resort.

## System Architecture & Data Model

The application uses a specific architectural pattern to handle complex, potentially changing tournament structures and database persistence.

### Data Structures

- **Composite Pattern**: A `Tournament` struct is composed of `TournamentBase`, `Stage`, `Group`, `Match`.
- **Identification**: All objects are identified by unique **UUIDs**.
- **Heritage**: Heritage is bottom-up; a child object knows its parent's ID (e.g., a Group knows its Stage ID).

### Separation of Concerns: Graph vs. Map

1. **Structure Graph (`petgraph`)**:
   - Represents the logical structure and dependencies (Tournament -> Stage -> Group).
   - Implemented as a Directed Acyclic Graph (DAG).
   - This is the "Substance" of the tournament structure.
2. **Data Storage (`HashMap`)**:
   - Stores the actual data objects, keyed by UUID.
   - Serves as the persistence layer representation.

### The "Orphan" Strategy

- **Definition**: Objects may exist in the `HashMaps` but generally be unreachable (disconnected) in the `structure` Graph.
- **Implementation**: When the tournament structure changes (e.g., reducing the number of groups), the edge in the graph is removed, but the object remains in the HashMap.
- **Reasoning**:
  - Prevents data loss during temporary configuration changes.
  - simplifies memory management in the reactive UI (Leptos). Unique components may hold references to objects; deleting them physically could cause race conditions or crashes in different parts of the UI state.
  - Memory overhead is considered negligible for typical tournament sizes.

### Validation

- **Input-Level Validation**: Validations are triggered immediately upon user input for individual objects (e.g., changing the Tournament Base name or Sport Configuration parameters).
- **Global Structure Validation**: A comprehensive `validate()` method exists on the `Tournament` struct to ensure the integrity and completeness of the entire tournament graph (e.g. checking if child objects match parent constraints). _Note: Completeness checks are currently under development._

### Data Persistence & Synchronization

- **Auto-Save**: The application follows an "Auto-Save" philosophy. Changes are persisted to the database immediately after validation passes.
- **Client Registry (WebSockets)**:
  - A registry system manages real-time updates across multiple clients.
  - Clients subscribe (register) to specific topics or object IDs.
  - When an object is created or modified, the server broadcasts the change only to clients registered for that specific event, ensuring efficient and fine-grained state synchronization.

## Software Architecture & Plugins

The application implements a **Hexagonal Architecture (Ports and Adapters)** realized within a Rust workspace. This design ensures separation of concerns, testability, and modularity.

### Core Architecture

- **Core (`app_core` crate)**:
  - Represents the center of the hexagon.
  - Contains all domain entities (Tournament, Stage, etc.), business logic, and rules.
  - Defines **Ports** as Rust traits (e.g., for storage, sport-specific logic).
  - Has **no dependencies** on external frameworks like database drivers or web servers.

- **Adapters (Outer Layers)**:
  - Implemented as separate crates in the workspace.
  - **Infrastructure Adapters**: Implement the ports defined in the core (e.g., `db_postgres` for persistence).
  - **Driving Adapters**: Consume the core domain logic (e.g., the web frontend application).
  - **Web UI**: Although part of the outer layer, the Web UI code interacts directly with both the Core domain entities and usage of Ports.

### Sport-Specific Plugins

To support various sports with unique requirements, the system uses a plugin architecture:

- **Concept**: Each sport acts as a plugin (Adapter) that implements specific logic defined by Ports in `app_core`.
- **Configuration**: Plugins define sport-specific configuration structures (e.g., field dimensions, scoring rules).
- **Web UI Integration**:
  - Plugins provide their own Web UI components (e.g., using `leptos` components).
  - These components are dynamically loaded and rendered by the main application to allow user interaction with sport-specific configurations (e.g., in `EditSportConfiguration` views).
  - Example: The DDC (Double Disc Court) plugin (`ddc_plugin`) implements its own configuration logic and UI forms, which are seamlessly integrated into the main app.
