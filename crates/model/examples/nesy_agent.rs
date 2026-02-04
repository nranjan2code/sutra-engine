/// Example: Neuro-Symbolic Agent
///
/// Demonstrates hybrid AI combining neural reasoning with symbolic tools
use sutra_nesy::{AgentConfig, NesyAgent};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Neuro-Symbolic AI Demo ===\n");

    // Configure agent
    let config = AgentConfig {
        max_steps: 10,
        verify_outputs: true,
        temperature: 0.7,
    };

    println!("Agent Configuration:");
    println!("  Max reasoning steps: {}", config.max_steps);
    println!("  Symbolic verification: {}", config.verify_outputs);

    // Create agent with built-in tools
    let agent = NesyAgent::new(config);

    println!("\n=== Architecture ===");
    println!("Neural Component:");
    println!("  - Small LLM (RWKV or Mamba)");
    println!("  - Understands queries");
    println!("  - Plans solutions");
    println!("  - Generates responses");

    println!("\nSymbolic Component:");
    println!("  - Calculator (exact arithmetic)");
    println!("  - Python executor (complex math)");
    println!("  - Logic solver (theorem proving)");
    println!("  - Verification engine");

    println!("\n=== Example Queries ===\n");

    // Example 1: Math
    println!("Query 1: 'What is 12345 * 67890?'");
    println!("Process:");
    println!("  1. Neural model identifies: needs calculation");
    println!("  2. Calls calculator tool: 12345 * 67890");
    println!("  3. Gets verified result: 838102050");
    println!("  4. Formats response: 'The result is 838,102,050'");
    println!("  ✓ Guaranteed correct (no hallucination)");

    // Example 2: Logic
    println!("\nQuery 2: 'Is (A ∧ B) → C equivalent to (¬A ∨ ¬B ∨ C)?'");
    println!("Process:");
    println!("  1. Neural model identifies: logical equivalence");
    println!("  2. Calls logic solver with formula");
    println!("  3. Solver proves: Yes, equivalent");
    println!("  4. Returns verified proof");
    println!("  ✓ Symbolically verified");

    // Example 3: Programming
    println!("\nQuery 3: 'Write code to compute Fibonacci(100)'");
    println!("Process:");
    println!("  1. Neural model generates Python code");
    println!("  2. Calls Python executor");
    println!("  3. Runs code safely");
    println!("  4. Returns actual result");
    println!("  ✓ Executed and verified");

    println!("\n=== Key Advantages ===");
    println!("✓ Reduced hallucinations via verification");
    println!("✓ Exact computation for math/logic");
    println!("✓ Smaller neural model sufficient");
    println!("✓ Antithesis to pure scaling");

    println!("\n=== Processing Query ===");
    let query = "What is 2 + 2?";
    let response = agent.process(query)?;

    println!("Query: {}", query);
    println!("Response: {}", response.text);
    println!("Verified: {}", response.verified);
    println!("Tool calls: {}", response.tool_calls);

    println!("\n=== Memory Footprint ===");
    println!("Total system:");
    println!("  - Small LLM (2-3GB quantized)");
    println!("  - Symbolic tools (<100MB)");
    println!("  - State (<10MB)");
    println!("  Total: ~3GB - easily fits in 16GB!");

    println!("\n=== Why This Works ===");
    println!("Traditional approach:");
    println!("  - Scale model to 100B+ params");
    println!("  - Hope it learns math/logic");
    println!("  - Still hallucinates");

    println!("\nNeSy approach:");
    println!("  - Use 3B param model for language");
    println!("  - Delegate math/logic to tools");
    println!("  - Verify symbolically");
    println!("  - Perfect accuracy + small size!");

    Ok(())
}
