# Overview

## Project Purpose & Vision

### Core Problem Statement

AI Context Loss & Setup Friction: Developers using AI coding assistants (specifically GitHub Copilot) face persistent challenges with context continuity across development sessions. Traditional approaches require manual context re-establishment, leading to:

- Context Amnesia: AI loses project understanding between sessions
- Manual Setup Overhead: Repeatedly explaining project structure, patterns, and current state
- Multi-Project Complexity: Managing context across related sub-projects within a workspace
- Knowledge Fragmentation: Development insights scattered across different sources

### Solution Architecture

Multi-Project Memory Bank Integration: airs-memspec serves as the bridge between structured development context (Multi-Project Memory Bank) and AI-assisted development workflows (GitHub Copilot). The tool provides:

- Custom Instructions Delivery: Seamlessly install Multi-Project Memory Bank custom instructions for GitHub Copilot
- Context State Reading: Parse and display current development state from memory bank structures
- Workspace Intelligence: Understand multi-project relationships and hierarchical context
- Development Flow Integration: Enable natural AI collaboration for memory bank maintenance