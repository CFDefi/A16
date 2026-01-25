# A16 Example Programs

> **Version:** 0.1 | **Status:** Draft | **Date:** 2026-01-19

---

## Example 1: Single Agent with Memory + Tools

A research assistant that remembers previous interactions and uses tools to search the web.

```a16
###
Research Assistant Agent
========================
A single agent that uses memory and tools to help with research tasks.
Demonstrates: agent definition, memory, tools, model invocation, token budgeting.
###

from a16.ai.agent import Agent
from a16.ai.model import gpt4
from a16.ai.memory import ShortTermMemory, LongTermMemory
from a16.ai.tools import web_search, file_read
from a16.data.json import parse

# Define output structures
struct SearchResult:
    title: Str
    url: Str
    snippet: Str

struct ResearchSummary:
    topic: Str
    key_findings: List[Str]
    sources: List[Str]
    confidence: Float
    follow_up_questions: List[Str]

# Define the research assistant agent
agent ResearchAssistant:
    """An intelligent research assistant with persistent memory."""
    
    # Model configuration
    model: gpt4
    
    # Memory configuration: short-term for recent context, long-term for knowledge
    memory: [
        ShortTermMemory(window=20),      # Last 20 interactions
        LongTermMemory(                   # Persistent knowledge base
            index_type="hnsw",
            dimensions=1536,
            capacity=100000
        )
    ]
    
    # Available tools
    tools: [web_search, file_read]
    
    # Budget constraints
    budget: tokens=10000, cost=0.50
    
    # Initialization
    fn __init__(self, name: Str = "Research Assistant"):
        self.name = name
        self.research_count = 0
    
    # Main research task
    async task research(topic: Str) -> ResearchSummary:
        """
        Research a topic using web search and memory.
        Returns a structured summary with sources.
        """
        
        # Check if we have relevant prior knowledge
        prior_knowledge = await memory.retrieve(topic, k=5)
        
        if prior_knowledge:
            print(f"Found {len(prior_knowledge)} relevant memories")
        
        # Search the web for current information
        print(f"Searching for: {topic}")
        search_results = await web_search(topic, num_results=10)
        
        # Build context from memory and search results
        context = self._build_context(prior_knowledge, search_results)
        
        # Generate structured summary using model
        summary = await model.structured(
            prompt=p"""
            You are a research assistant analyzing information about: {topic}
            
            Previous Knowledge:
            {self._format_memories(prior_knowledge)}
            
            Current Search Results:
            {self._format_results(search_results)}
            
            Provide a comprehensive research summary with:
            1. Key findings (3-5 bullet points)
            2. Sources used
            3. Confidence level (0-1)
            4. Suggested follow-up questions
            """,
            schema=ResearchSummary
        )
        
        # Store the research in long-term memory
        await memory.store(
            content=f"Research on {topic}: {summary.key_findings}",
            metadata={
                "type": "research",
                "topic": topic,
                "confidence": summary.confidence,
                "timestamp": now()
            }
        )
        
        self.research_count += 1
        return summary
    
    # Follow-up question handler
    async task follow_up(question: Str) -> Str:
        """Handle follow-up questions using context from memory."""
        
        # Retrieve relevant context
        context = await memory.retrieve(question, k=10)
        
        response = await model.generate(
            prompt=p"""
            Based on our previous research:
            {self._format_memories(context)}
            
            User question: {question}
            
            Provide a helpful, accurate response.
            """
        )
        
        # Store the interaction
        await memory.store(f"Q: {question}\nA: {response}")
        
        return response
    
    # Helper methods
    fn _build_context(self, memories: List[Any], results: List[SearchResult]) -> Str:
        parts = []
        
        if memories:
            parts.append("## Prior Knowledge")
            for m in memories:
                parts.append(f"- {m.content}")
        
        if results:
            parts.append("## Search Results")
            for r in results:
                parts.append(f"- [{r.title}]({r.url}): {r.snippet}")
        
        return "\n".join(parts)
    
    fn _format_memories(self, memories: List[Any]) -> Str:
        if not memories:
            return "No prior knowledge available."
        return "\n".join(f"- {m.content}" for m in memories)
    
    fn _format_results(self, results: List[SearchResult]) -> Str:
        if not results:
            return "No search results available."
        return "\n".join(f"- {r.title}: {r.snippet}" for r in results)


# Main program
async fn main():
    # Create the assistant
    let assistant = ResearchAssistant(name="Alex")
    
    print(f"Initialized {assistant.name}")
    print(f"Token budget: {assistant.budget.tokens}")
    print()
    
    # Research a topic
    let topic = "latest developments in quantum computing 2024"
    print(f"Researching: {topic}")
    print("-" * 50)
    
    let summary = await assistant.research(topic)
    
    print(f"\n📊 Research Summary: {summary.topic}")
    print(f"Confidence: {summary.confidence:.0%}")
    print("\nKey Findings:")
    for finding in summary.key_findings:
        print(f"  • {finding}")
    
    print("\nSources:")
    for source in summary.sources:
        print(f"  📎 {source}")
    
    print("\nSuggested Follow-ups:")
    for question in summary.follow_up_questions:
        print(f"  ❓ {question}")
    
    # Ask a follow-up question
    print("\n" + "=" * 50)
    let follow_up_q = summary.follow_up_questions[0]
    print(f"Follow-up: {follow_up_q}")
    
    let answer = await assistant.follow_up(follow_up_q)
    print(f"\n💬 Answer:\n{answer}")
    
    # Show token usage
    print("\n" + "=" * 50)
    print(f"Total researches: {assistant.research_count}")
    print(f"Tokens used: {assistant.budget.used}/{assistant.budget.tokens}")
    print(f"Cost: ${assistant.budget.cost_used:.4f}")


# Run the program
if __name__ == "__main__":
    run(main())
```

---

## Example 2: Multi-Agent Team (Planner/Executor/Critic)

A team of agents that collaborate to complete complex tasks with planning, execution, and review.

```a16
###
Multi-Agent Research Team
=========================
A coordinated team of three agents: Planner, Executor, and Critic.
Demonstrates: multi-agent coordination, message passing, shared memory, team topology.
###

from a16.ai.agent import Agent, Team, Message
from a16.ai.model import gpt4, gpt4_mini
from a16.ai.memory import SharedMemory
from a16.ai.tools import web_search, code_execute, file_write

# Shared structures
struct TaskPlan:
    goal: Str
    steps: List[PlanStep]
    success_criteria: List[Str]
    estimated_time: Str

struct PlanStep:
    id: Int
    action: Str
    tool: Optional[Str]
    dependencies: List[Int]
    expected_output: Str

struct ExecutionResult:
    step_id: Int
    success: Bool
    output: Any
    error: Optional[Str]

struct CritiqueReport:
    approved: Bool
    score: Float  # 0-10
    strengths: List[Str]
    issues: List[Str]
    suggestions: List[Str]
    verdict: Str


# Agent 1: The Planner
agent Planner:
    """Creates detailed plans for complex tasks."""
    
    model: gpt4
    memory: [SharedMemory("team_memory")]
    budget: tokens=3000
    
    async task create_plan(goal: Str, context: Optional[Str] = None) -> TaskPlan:
        """Create a detailed execution plan for the given goal."""
        
        # Check for similar past plans
        past_plans = await memory.retrieve(f"plan: {goal}", k=3)
        
        plan = await model.structured(
            prompt=p"""
            You are an expert planner. Create a detailed plan to achieve:
            
            GOAL: {goal}
            
            {% if context %}
            ADDITIONAL CONTEXT:
            {context}
            {% endif %}
            
            {% if past_plans %}
            SIMILAR PAST PLANS (for reference):
            {% for p in past_plans %}
            - {p.content}
            {% endfor %}
            {% endif %}
            
            Requirements:
            1. Break down into clear, actionable steps
            2. Identify which tools to use for each step
            3. Define dependencies between steps
            4. Specify success criteria
            5. Be thorough but efficient
            """,
            schema=TaskPlan
        )
        
        # Store plan in shared memory
        await memory.store(
            content=f"plan: {goal} -> {len(plan.steps)} steps",
            metadata={"type": "plan", "goal": goal}
        )
        
        return plan
    
    async task revise_plan(original: TaskPlan, feedback: CritiqueReport) -> TaskPlan:
        """Revise a plan based on critic feedback."""
        
        revised = await model.structured(
            prompt=p"""
            Revise this plan based on the critique:
            
            ORIGINAL PLAN:
            {original}
            
            CRITIQUE:
            Score: {feedback.score}/10
            Issues: {feedback.issues}
            Suggestions: {feedback.suggestions}
            
            Create an improved plan addressing the feedback.
            """,
            schema=TaskPlan
        )
        
        return revised


# Agent 2: The Executor
agent Executor:
    """Executes plans step by step using available tools."""
    
    model: gpt4_mini  # Faster model for execution
    memory: [SharedMemory("team_memory")]
    tools: [web_search, code_execute, file_write]
    budget: tokens=5000
    
    async task execute_plan(plan: TaskPlan) -> List[ExecutionResult]:
        """Execute all steps in the plan."""
        
        results = []
        completed_steps = {}
        
        for step in plan.steps:
            # Check dependencies
            deps_met = all(
                completed_steps.get(dep_id, False) 
                for dep_id in step.dependencies
            )
            
            if not deps_met:
                results.append(ExecutionResult(
                    step_id=step.id,
                    success=False,
                    output=None,
                    error="Dependencies not met"
                ))
                continue
            
            # Execute the step
            result = await self._execute_step(step, completed_steps)
            results.append(result)
            completed_steps[step.id] = result.success
            
            # Store progress
            await memory.store(
                content=f"executed step {step.id}: {result.success}",
                metadata={"type": "execution", "step_id": step.id}
            )
        
        return results
    
    async fn _execute_step(
        self, 
        step: PlanStep, 
        prior_results: Dict[Int, Bool]
    ) -> ExecutionResult:
        """Execute a single step."""
        
        try:
            if step.tool:
                # Use specified tool
                tool_fn = self.tools[step.tool]
                
                # Let model decide tool parameters
                params = await model.structured(
                    prompt=p"""
                    Determine parameters for this tool call:
                    
                    Step: {step.action}
                    Tool: {step.tool}
                    Expected output: {step.expected_output}
                    
                    Return the tool parameters.
                    """,
                    schema=tool_fn.param_schema
                )
                
                output = await tool_fn(**params)
            else:
                # Pure reasoning step
                output = await model.generate(
                    prompt=p"""
                    Complete this step:
                    {step.action}
                    
                    Expected output: {step.expected_output}
                    """
                )
            
            return ExecutionResult(
                step_id=step.id,
                success=True,
                output=output,
                error=None
            )
            
        except Exception as e:
            return ExecutionResult(
                step_id=step.id,
                success=False,
                output=None,
                error=str(e)
            )


# Agent 3: The Critic
agent Critic:
    """Reviews plans and execution results for quality."""
    
    model: gpt4
    memory: [SharedMemory("team_memory")]
    budget: tokens=2000
    
    async task review_plan(plan: TaskPlan, goal: Str) -> CritiqueReport:
        """Review a plan before execution."""
        
        critique = await model.structured(
            prompt=p"""
            Critically review this plan:
            
            GOAL: {goal}
            
            PLAN:
            Steps: {len(plan.steps)}
            {self._format_plan(plan)}
            
            Evaluate:
            1. Completeness: Does it cover all aspects of the goal?
            2. Feasibility: Are all steps achievable?
            3. Efficiency: Is it the most efficient approach?
            4. Risk: What could go wrong?
            
            Be constructive but thorough. Score 1-10.
            """,
            schema=CritiqueReport
        )
        
        return critique
    
    async task review_execution(
        plan: TaskPlan, 
        results: List[ExecutionResult],
        goal: Str
    ) -> CritiqueReport:
        """Review execution results."""
        
        success_rate = sum(1 for r in results if r.success) / len(results)
        
        critique = await model.structured(
            prompt=p"""
            Review this execution:
            
            GOAL: {goal}
            SUCCESS RATE: {success_rate:.0%}
            
            RESULTS:
            {% for r in results %}
            Step {r.step_id}: {"✓" if r.success else "✗"} 
            {% if r.error %}Error: {r.error}{% endif %}
            {% endfor %}
            
            Evaluate:
            1. Did execution achieve the goal?
            2. What worked well?
            3. What failed and why?
            4. Recommendations for improvement?
            """,
            schema=CritiqueReport
        )
        
        return critique
    
    fn _format_plan(self, plan: TaskPlan) -> Str:
        lines = []
        for step in plan.steps:
            deps = f" (depends on: {step.dependencies})" if step.dependencies else ""
            tool = f" [tool: {step.tool}]" if step.tool else ""
            lines.append(f"{step.id}. {step.action}{tool}{deps}")
        return "\n".join(lines)


# Team Coordinator
team ResearchTeam:
    """A coordinated team of Planner, Executor, and Critic."""
    
    agents: [Planner, Executor, Critic]
    memory: SharedMemory("team_memory")
    topology: "sequential"  # Agents work in sequence
    max_iterations: 3
    
    async fn run(goal: Str) -> Dict:
        """Execute a goal using the full team."""
        
        print(f"🎯 Team Goal: {goal}")
        print("=" * 60)
        
        # Phase 1: Planning
        print("\n📋 Phase 1: Planning")
        plan = await Planner.create_plan(goal)
        print(f"Created plan with {len(plan.steps)} steps")
        
        # Phase 2: Plan Review
        print("\n🔍 Phase 2: Plan Review")
        plan_critique = await Critic.review_plan(plan, goal)
        print(f"Plan score: {plan_critique.score}/10")
        
        # Iterate if plan needs improvement
        iteration = 0
        while not plan_critique.approved and iteration < self.max_iterations:
            print(f"\n🔄 Iteration {iteration + 1}: Revising plan...")
            plan = await Planner.revise_plan(plan, plan_critique)
            plan_critique = await Critic.review_plan(plan, goal)
            print(f"Revised plan score: {plan_critique.score}/10")
            iteration += 1
        
        if not plan_critique.approved:
            print("⚠️ Plan not fully approved, proceeding with best effort")
        
        # Phase 3: Execution
        print("\n⚡ Phase 3: Execution")
        results = await Executor.execute_plan(plan)
        
        success_count = sum(1 for r in results if r.success)
        print(f"Completed: {success_count}/{len(results)} steps successful")
        
        # Phase 4: Execution Review
        print("\n📊 Phase 4: Execution Review")
        exec_critique = await Critic.review_execution(plan, results, goal)
        print(f"Execution score: {exec_critique.score}/10")
        print(f"Verdict: {exec_critique.verdict}")
        
        return {
            "goal": goal,
            "plan": plan,
            "results": results,
            "plan_critique": plan_critique,
            "execution_critique": exec_critique,
            "iterations": iteration + 1
        }


# Main program
async fn main():
    # Create the team
    let team = ResearchTeam()
    
    # Define a complex goal
    let goal = """
    Research and create a comprehensive report on the current state of 
    renewable energy adoption worldwide, including:
    1. Top 5 countries by renewable energy percentage
    2. Key trends in solar and wind energy
    3. Major challenges facing adoption
    4. Predictions for 2030
    Save the report to 'renewable_energy_report.md'
    """
    
    # Run the team
    let result = await team.run(goal)
    
    # Summary
    print("\n" + "=" * 60)
    print("📈 MISSION SUMMARY")
    print("=" * 60)
    print(f"Goal achieved: {result['execution_critique'].approved}")
    print(f"Plan iterations: {result['iterations']}")
    print(f"Final score: {result['execution_critique'].score}/10")
    
    if result['execution_critique'].suggestions:
        print("\n💡 Suggestions for next time:")
        for s in result['execution_critique'].suggestions:
            print(f"  • {s}")


if __name__ == "__main__":
    run(main())
```

---

## Example 3: Long-Running Autonomous Workflow

An event-driven autonomous workflow that monitors, processes, and acts on incoming data.

```a16
###
Autonomous Data Processing Workflow
===================================
A long-running, event-driven workflow that monitors data sources,
processes information, and takes automated actions.
Demonstrates: event handling, persistent state, rate limiting, 
error recovery, graceful shutdown.
###

from a16.ai.agent import Agent
from a16.ai.model import gpt4_mini
from a16.ai.memory import LongTermMemory, EpisodicMemory
from a16.ai.tools import web_fetch, email_send, slack_post, database_query
from a16.sys.concurrent import Channel, spawn, select, sleep
from a16.sys.io import File
from a16.sys.time import now, Duration

# Event types
enum EventType:
    NewData
    ScheduledCheck
    Alert
    UserCommand
    Shutdown

struct Event:
    type: EventType
    payload: Any
    timestamp: DateTime
    source: Str

struct ProcessingResult:
    event_id: Str
    success: Bool
    action_taken: Str
    insights: List[Str]
    next_check: Optional[Duration]

struct Alert:
    severity: Str  # low, medium, high, critical
    message: Str
    data: Any
    recommended_action: Str

struct WorkflowState:
    started_at: DateTime
    events_processed: Int
    alerts_generated: Int
    errors: Int
    last_activity: DateTime
    status: Str


# The autonomous workflow agent
agent DataMonitor:
    """
    An autonomous agent that continuously monitors data sources,
    detects anomalies, and takes appropriate actions.
    """
    
    model: gpt4_mini
    
    memory: [
        LongTermMemory(capacity=100000),  # Historical data
        EpisodicMemory(                    # Recent events
            retention=7d,
            compression="summarize"
        )
    ]
    
    tools: [web_fetch, email_send, slack_post, database_query]
    
    budget: tokens=50000, cost=10.0  # Daily budget
    
    # State
    state: WorkflowState
    event_channel: Channel[Event]
    alert_channel: Channel[Alert]
    running: Bool = True
    
    fn __init__(self, config: Dict):
        self.config = config
        self.event_channel = Channel[Event](buffer=100)
        self.alert_channel = Channel[Alert](buffer=50)
        self.state = WorkflowState(
            started_at=now(),
            events_processed=0,
            alerts_generated=0,
            errors=0,
            last_activity=now(),
            status="initializing"
        )
    
    # Main run loop
    async fn run(self):
        """Main event processing loop."""
        
        print(f"🚀 DataMonitor starting at {self.state.started_at}")
        self.state.status = "running"
        
        # Start background tasks
        spawn self._scheduled_checks()
        spawn self._alert_handler()
        spawn self._health_reporter()
        
        # Main event loop
        while self.running:
            try:
                match await select(
                    self.event_channel.receive(),
                    sleep(60s)  # Check every minute even if no events
                ):
                    case (0, event):
                        await self._process_event(event)
                    case (1, _):
                        # Timeout - do periodic maintenance
                        await self._maintenance()
                        
            except Exception as e:
                self.state.errors += 1
                await self._handle_error(e)
        
        print(f"🛑 DataMonitor shutting down after {self.state.events_processed} events")
    
    # Event processing
    async fn _process_event(self, event: Event):
        """Process a single event."""
        
        self.state.last_activity = now()
        self.state.events_processed += 1
        
        print(f"📥 Processing event: {event.type} from {event.source}")
        
        match event.type:
            case EventType.NewData:
                await self._handle_new_data(event)
            case EventType.Alert:
                await self._handle_alert(event)
            case EventType.ScheduledCheck:
                await self._handle_scheduled_check(event)
            case EventType.UserCommand:
                await self._handle_command(event)
            case EventType.Shutdown:
                self.running = False
    
    # Data analysis
    async fn _handle_new_data(self, event: Event):
        """Analyze new data and detect anomalies."""
        
        data = event.payload
        
        # Retrieve historical context
        history = await memory.retrieve(
            query=f"data from {event.source}",
            k=10
        )
        
        # Analyze with AI
        analysis = await model.structured(
            prompt=p"""
            Analyze this incoming data for anomalies:
            
            SOURCE: {event.source}
            DATA: {data}
            
            HISTORICAL CONTEXT:
            {self._format_history(history)}
            
            Check for:
            1. Unusual patterns
            2. Threshold violations
            3. Trend changes
            4. Missing expected data
            
            Return analysis with severity and recommended action.
            """,
            schema=ProcessingResult
        )
        
        # Store in memory
        await memory.store(
            content=f"Data from {event.source}: {analysis.insights}",
            metadata={
                "source": event.source,
                "timestamp": event.timestamp,
                "anomaly": analysis.action_taken != "none"
            }
        )
        
        # Generate alert if needed
        if "alert" in analysis.action_taken.lower():
            await self.alert_channel.send(Alert(
                severity=self._determine_severity(analysis),
                message=analysis.insights[0] if analysis.insights else "Anomaly detected",
                data=data,
                recommended_action=analysis.action_taken
            ))
        
        # Schedule follow-up if needed
        if analysis.next_check:
            spawn self._schedule_follow_up(event.source, analysis.next_check)
    
    # Alert handling
    async fn _alert_handler(self):
        """Background task that processes alerts."""
        
        async for alert in self.alert_channel:
            self.state.alerts_generated += 1
            
            print(f"🚨 Alert [{alert.severity.upper()}]: {alert.message}")
            
            # Route based on severity
            match alert.severity:
                case "critical":
                    await self._notify_critical(alert)
                case "high":
                    await self._notify_high(alert)
                case "medium":
                    await self._notify_medium(alert)
                case "low":
                    await self._log_alert(alert)
    
    async fn _notify_critical(self, alert: Alert):
        """Handle critical alerts - multiple notification channels."""
        
        # Send email
        await email_send(
            to=self.config["critical_email"],
            subject=f"🚨 CRITICAL: {alert.message}",
            body=self._format_alert_email(alert)
        )
        
        # Post to Slack
        await slack_post(
            channel=self.config["alerts_channel"],
            message=f"🚨 *CRITICAL ALERT*\n{alert.message}\n\n*Action:* {alert.recommended_action}"
        )
        
        # Log to database
        await database_query(f"""
            INSERT INTO alerts (severity, message, data, created_at)
            VALUES ('critical', '{alert.message}', '{alert.data}', NOW())
        """)
    
    # Scheduled checks
    async fn _scheduled_checks(self):
        """Run periodic health checks and data polls."""
        
        while self.running:
            await sleep(self.config.get("check_interval", 5m))
            
            for source in self.config["data_sources"]:
                try:
                    data = await web_fetch(source["url"])
                    await self.event_channel.send(Event(
                        type=EventType.NewData,
                        payload=data,
                        timestamp=now(),
                        source=source["name"]
                    ))
                except Exception as e:
                    print(f"⚠️ Failed to fetch {source['name']}: {e}")
    
    # Health reporting
    async fn _health_reporter(self):
        """Periodic health status reporting."""
        
        while self.running:
            await sleep(1h)
            
            uptime = now() - self.state.started_at
            
            report = f"""
            📊 DataMonitor Health Report
            ═══════════════════════════════
            Status: {self.state.status}
            Uptime: {uptime}
            Events processed: {self.state.events_processed}
            Alerts generated: {self.state.alerts_generated}
            Errors: {self.state.errors}
            Token budget: {self.budget.used}/{self.budget.tokens} ({self.budget.percentage_used:.1%})
            """
            
            print(report)
            
            # Save state snapshot
            await File.write_json(
                ".a16/state_snapshot.json",
                self.state
            )
    
    # Maintenance
    async fn _maintenance(self):
        """Periodic maintenance tasks."""
        
        # Compress old memories
        if self.state.events_processed % 100 == 0:
            await memory.compress(older_than=7d)
        
        # Check budget
        if self.budget.percentage_used > 0.8:
            print("⚠️ Token budget at 80%, reducing activity")
            self.config["check_interval"] *= 2
    
    # Error handling with recovery
    async fn _handle_error(self, error: Exception):
        """Handle errors with exponential backoff."""
        
        print(f"❌ Error: {error}")
        
        # Log to memory for pattern detection
        await memory.store(
            content=f"Error: {error}",
            metadata={"type": "error", "timestamp": now()}
        )
        
        # Check for repeated errors
        recent_errors = await memory.retrieve(
            query="type:error",
            k=10,
            filter={"timestamp": {"$gte": now() - 1h}}
        )
        
        if len(recent_errors) > 5:
            await self.alert_channel.send(Alert(
                severity="high",
                message=f"High error rate detected: {len(recent_errors)} errors in last hour",
                data={"errors": [e.content for e in recent_errors]},
                recommended_action="Investigate and possibly restart service"
            ))
    
    # Command handler (for external control)
    async fn _handle_command(self, event: Event):
        """Handle external commands."""
        
        command = event.payload
        
        match command.get("action"):
            case "status":
                return self.state
            case "pause":
                self.state.status = "paused"
            case "resume":
                self.state.status = "running"
            case "shutdown":
                self.running = False
            case "compress_memory":
                await memory.compress()
            case "reset_budget":
                self.budget.reset()
    
    # Helpers
    fn _format_history(self, history: List[Any]) -> Str:
        if not history:
            return "No historical data available"
        return "\n".join(f"- {h.content}" for h in history[-5:])
    
    fn _determine_severity(self, analysis: ProcessingResult) -> Str:
        # Simple heuristic - could be more sophisticated
        action = analysis.action_taken.lower()
        if "critical" in action or "immediate" in action:
            return "critical"
        elif "urgent" in action or "high" in action:
            return "high"
        elif "warning" in action:
            return "medium"
        return "low"
    
    fn _format_alert_email(self, alert: Alert) -> Str:
        return f"""
        Alert Severity: {alert.severity.upper()}
        
        Message: {alert.message}
        
        Data:
        {alert.data}
        
        Recommended Action:
        {alert.recommended_action}
        
        ---
        Generated by A16 DataMonitor
        """


# Configuration
const CONFIG = {
    "data_sources": [
        {"name": "api-metrics", "url": "https://api.example.com/metrics"},
        {"name": "sales-data", "url": "https://data.example.com/sales"},
        {"name": "system-health", "url": "https://monitor.example.com/health"}
    ],
    "check_interval": 5m,
    "critical_email": "oncall@example.com",
    "alerts_channel": "#alerts",
    "error_threshold": 5
}


# Main entry point
async fn main():
    let monitor = DataMonitor(CONFIG)
    
    # Handle graceful shutdown
    on_signal(SIGTERM, async () => {
        print("\n🛑 Shutdown signal received")
        await monitor.event_channel.send(Event(
            type=EventType.Shutdown,
            payload=None,
            timestamp=now(),
            source="system"
        ))
    })
    
    # Run the monitor
    await monitor.run()
    
    # Final report
    print("\n📊 Final Statistics:")
    print(f"  Total events: {monitor.state.events_processed}")
    print(f"  Total alerts: {monitor.state.alerts_generated}")
    print(f"  Total errors: {monitor.state.errors}")
    print(f"  Tokens used: {monitor.budget.used}")


if __name__ == "__main__":
    run(main())
```

---

## Running the Examples

```bash
# Example 1: Research Assistant
a16 run examples/research_assistant.a16

# Example 2: Multi-Agent Team
a16 run examples/research_team.a16

# Example 3: Autonomous Workflow (long-running)
a16 run examples/data_monitor.a16

# Run with debugging
a16 run --debug examples/research_assistant.a16

# Run with token tracking
a16 run --profile examples/research_team.a16

# Run in deterministic mode (for testing)
a16 run --deterministic --seed=42 examples/research_assistant.a16
```

---

*End of A16 Specification Documents*
