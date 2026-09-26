<script setup lang="ts">
import ShelfArt from './ShelfArt.vue'

const situations = [
  'A context window is filling up, and there’s an insight worth keeping for a session you haven’t started.',
  'You built a nice component before realizing the task no longer needs it.',
  'Research done in a throwaway worktree needs to reach a session in a different one.',
  'Design documents should inform implementation across sessions and repositories.',
]

const ideas = [
  {
    label: 'Outlive',
    title: 'Information outlives the session',
    body: 'Keep a note, a handoff, a design doc, or a piece of code independently of the conversation, repository, or worktree it came from.',
  },
  {
    label: 'Share',
    title: 'People and agents use the same material',
    body: 'A shelf is ordinary Markdown in a directory you own. You browse and edit it in your editor; agents use the same bs commands with JSON output.',
  },
  {
    label: 'Shape',
    title: 'Each shelf has its own conventions',
    body: 'A shelf can describe, in plain language, how its contents should be written, updated, and found—and enforce structure such as required tags.',
  },
]

const retrieval = [
  { cmd: 'bs shelf list', text: 'Discover which shelves exist and what each is for.' },
  { cmd: 'bs context SHELF', text: 'Load a shelf’s guidance only when it’s relevant.' },
  { cmd: 'bs search QUERY', text: 'Find candidates by ID, title, tags, and body.' },
  { cmd: 'bs show ID', text: 'Read just the bits that were chosen.' },
]

const adds = [
  { title: 'Conventions agents actually load', body: 'bs context hands an agent a shelf’s full guidance and requirements before it writes.' },
  { title: 'Structure that’s checked', body: 'Shelves can require titles or tags and restrict tag values, for example to known repository names. Invalid saves are rejected.' },
  { title: 'Stable identifiers and JSON', body: 'Every bit is shelf/name, and every command has structured output for agents and scripts.' },
  { title: 'Exact content', body: 'Prompts and code are stored byte-for-byte; metadata lives in frontmatter.' },
  { title: 'Discovery you control', body: 'An archive shelf can stay out of default results while remaining accessible.' },
]

const recipes = [
  { title: 'Design docs ready for implementation', body: 'Import, tag, and refresh reference material that agents implement against', link: '/recipes/design-docs' },
  { title: 'Focused handoffs between sessions', body: 'Brief a fresh session without re-explaining everything', link: '/recipes/handoffs' },
  { title: 'Keep code that didn’t ship', body: 'Save a component with enough context to adapt it in a later task', link: '/recipes/reusable-code' },
  { title: 'Archive without deleting', body: 'Move finished material out of everyday results', link: '/recipes/archive' },
  { title: 'Let scratch material expire', body: 'Give temporary shelves a retention period', link: '/recipes/expiring-shelves' },
]
</script>

<template>
  <div class="bs-home">
    <!-- Hero ---------------------------------------------------------------- -->
    <section class="hero">
      <div class="wrap hero-grid">
        <div class="hero-copy">
          <p class="eyebrow"><span class="caret">$</span> man bs <span class="dim">— a shelf for work worth keeping</span></p>
          <h1 class="hero-title">
            A home for work that outlives the session.
          </h1>
          <p class="hero-tagline">Notes, snippets, prompts, design docs, and handoffs—kept on your machine, shared by people and agents.</p>
          <p class="hero-body">
            Agents let us pursue more work in parallel: more sessions, more worktrees, more repositories.
            bitshelf gives the work you choose to keep a common home outside any one of them—ordinary
            Markdown files with conventions that both you and your agents can follow.
          </p>
          <div class="actions">
            <a class="btn primary" href="/start">Get started <span aria-hidden="true">→</span></a>
            <a class="btn secondary" href="/agents">Use with agents</a>
          </div>
          <p class="hero-sub">Know what you need? <a href="/recipes/design-docs">Browse the recipes →</a></p>
          <ul class="facts" aria-label="Design">
            <li>Markdown</li><li>JSON output</li><li>No database</li><li>No daemon</li>
          </ul>
        </div>

        <div class="hero-art">
          <div class="art-frame">
            <ShelfArt />
          </div>
          <figure class="window term hero-term" aria-label="Terminal session showing the core loop">
            <div class="window-bar"><span class="close-box" aria-hidden="true" /><span class="window-title">zsh — ~/src</span></div>
<pre class="window-body"><code><span class="c"># session A, in a research worktree</span>
<span class="p">api-wt $</span> bs add notes/pagination-research \
    --tags api --file findings.md
<span class="o">notes/pagination-research</span>

<span class="c"># three days later, another repository</span>
<span class="p">web $</span> bs search pagination
<span class="o">notes/pagination-research</span>
<span class="p">web $</span> bs show notes/pagination-research --body
<span class="o"># Cursor pagination</span>
<span class="o">Use opaque cursors; page size 50.</span>
<span class="p">web $</span> <span class="cursor" aria-hidden="true"></span></code></pre>
          </figure>
        </div>
      </div>
      <p class="wrap prototype-note">
        <strong>Prototype.</strong> Commands, configuration, and file formats may change without backward
        compatibility, and bugs may cause data loss. Use disposable data. This site tracks <code>main</code>
        and may describe unreleased work.
      </p>
    </section>

    <!-- Problem ------------------------------------------------------------- -->
    <section class="section wrap problem">
      <div class="problem-lead">
        <p class="label">The problem</p>
        <h2>Useful work now outlives the conversation that produced it—and it’s easy to lose.</h2>
      </div>
      <ul class="situations">
        <li v-for="(s, i) in situations" :key="i">
          <span class="num">{{ String(i + 1).padStart(2, '0') }}</span>
          <span>{{ s }}</span>
        </li>
      </ul>
      <p class="problem-answer">
        <strong>bitshelf gives that work a home outside any one session.</strong>
        You or your agent deliberately put something on a shelf; later, from any context, either of you
        finds it, reads it, and builds on it. The executable is <code>bs</code>.
      </p>
    </section>

    <!-- Three ideas --------------------------------------------------------- -->
    <section class="section wrap">
      <h2 class="section-title">Three ideas</h2>
      <div class="ideas">
        <article v-for="(idea, i) in ideas" :key="idea.label" class="idea">
          <p class="label">{{ String(i + 1).padStart(2, '0') }} / {{ idea.label }}</p>
          <h3>{{ idea.title }}</h3>
          <p>{{ idea.body }}</p>
        </article>
      </div>
    </section>

    <!-- The loop ------------------------------------------------------------ -->
    <section class="section wrap loop">
      <div>
        <p class="label">The loop</p>
        <h2>Keep it. Find it from somewhere else. Use it again.</h2>
        <p>
          Save something in one session. Days later, in another worktree or repository, find it,
          read it, and build on it rather than duplicating it. The second use is the payoff.
        </p>
        <p>With the agent skill installed, the same loop is a request:</p>
        <div class="requests">
          <p class="request"><span class="ctx">session A</span>“save these pagination findings to bs”</p>
          <p class="request"><span class="ctx">session B</span>“check bs for our pagination research”</p>
        </div>
        <a class="more" href="/start">Walk through the loop →</a>
      </div>
      <div class="retrieval">
        <div class="retrieval-head">
          <p class="label">Deliberate, not automatic memory</p>
          <p>
            Nothing is saved unless you, or an agent you asked, saves it. Nothing is loaded into a
            conversation unless it’s looked up. Agents disclose the store to themselves progressively:
          </p>
        </div>
        <ol>
          <li v-for="(step, i) in retrieval" :key="step.cmd">
            <span class="step">{{ i + 1 }}</span>
            <div>
              <code>{{ step.cmd }}</code>
              <p>{{ step.text }}</p>
            </div>
          </li>
        </ol>
        <p class="retrieval-foot">You stay in control of what’s kept, and what’s kept stays inspectable.</p>
      </div>
    </section>

    <!-- Why not a folder ---------------------------------------------------- -->
    <section class="section wrap folder">
      <div class="folder-copy">
        <p class="label">Why not just a folder of Markdown?</p>
        <h2>It <em>is</em> a folder of Markdown.</h2>
        <p>
          Browse it, grep it, back it up, or commit it to Git, with or without <code>bs</code>.
          No account, database, or background service. What <code>bs</code> adds is what makes that
          folder dependable for both people and agents.
        </p>
        <figure class="window tree" aria-label="Directory tree of a bitshelf store">
          <div class="window-bar"><span class="close-box" aria-hidden="true" /><span class="window-title">~/bitshelf</span></div>
<pre class="window-body"><code><span class="d">~/bitshelf/</span>
├── <span class="d">notes/</span>
│   ├── bs.toml         <span class="c"># settings</span>
│   └── <span class="d">bits/</span>
│       └── pagination-research.md
└── <span class="d">handoffs/</span>
    ├── bs.toml
    ├── SHELF.md        <span class="c"># conventions</span>
    └── <span class="d">bits/</span>
        └── docs-rewrite.md</code></pre>
        </figure>
      </div>
      <dl class="adds">
        <div v-for="a in adds" :key="a.title" class="add">
          <dt>{{ a.title }}</dt>
          <dd>{{ a.body }}</dd>
        </div>
      </dl>
    </section>

    <!-- Recipes ------------------------------------------------------------- -->
    <section class="section wrap">
      <div class="panel">
        <div class="panel-copy">
          <p class="label">Recipes</p>
          <h2>Shape shelves around the work you repeat.</h2>
          <p>
            Each recipe starts from an outcome, shows the finished experience, and separates what
            <code>bs</code> enforces from what a shelf’s conventions ask of the people and agents using it.
          </p>
          <a class="more" href="/agents">How agents use shelves →</a>
        </div>
        <ul class="recipe-list">
          <li v-for="rc in recipes" :key="rc.link">
            <a :href="rc.link">
              <span class="recipe-title">{{ rc.title }}</span>
              <span class="recipe-body">{{ rc.body }} <span aria-hidden="true">→</span></span>
            </a>
          </li>
        </ul>
      </div>
    </section>

    <section class="section wrap next">
      <a href="/start"><span class="label">Start</span>Get started<span class="arrow">→</span></a>
      <a href="/agents"><span class="label">Agents</span>Using with agents<span class="arrow">→</span></a>
      <a href="/install"><span class="label">Install</span>Install bs<span class="arrow">→</span></a>
      <a href="/concepts/shelves"><span class="label">Reference</span>Behavior and limits<span class="arrow">→</span></a>
    </section>
  </div>
</template>

<style scoped>
.bs-home {
  --wrap: 1180px;
  padding-bottom: 32px;
}

.wrap {
  max-width: var(--wrap);
  margin: 0 auto;
  padding: 0 24px;
}

@media (min-width: 768px) {
  .wrap { padding: 0 40px; }
}

code {
  font-family: var(--vp-font-family-mono);
  font-size: 0.88em;
  padding: 1px 5px;
  border: 1px solid var(--bs-rule);
  border-radius: 2px;
  background: var(--bs-paper-3);
}

a { color: var(--bs-amber); }

.label {
  margin: 0 0 14px;
  font-family: var(--vp-font-family-mono);
  font-size: 12px;
  font-weight: 500;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--bs-amber);
}

h2 {
  margin: 0;
  font-size: clamp(1.6rem, 2.6vw, 2.1rem);
  line-height: 1.2;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--bs-ink);
}

p {
  margin: 0 0 14px;
  color: var(--bs-ink-2);
  line-height: 1.7;
}

.more {
  display: inline-block;
  margin-top: 8px;
  font-weight: 500;
  text-decoration: underline;
  text-decoration-color: color-mix(in srgb, currentColor 35%, transparent);
  text-underline-offset: 4px;
}

.more:hover { text-decoration-color: currentColor; }

.section { margin-top: 112px; }

/* Let grid columns shrink below their content (wide <pre> blocks scroll instead). */
.hero-grid > *,
.problem > *,
.loop > *,
.folder > *,
.panel > * { min-width: 0; }

.section-title {
  padding-bottom: 20px;
}

/* --- Hero --------------------------------------------------------------- */

.hero {
  position: relative;
  padding: 56px 0 0;
  border-bottom: 1px solid var(--bs-rule);
  background:
    linear-gradient(to bottom, transparent 70%, var(--bs-paper-2)),
    repeating-linear-gradient(to bottom, transparent 0 31px, color-mix(in srgb, var(--bs-rule) 45%, transparent) 31px 32px);
}

.hero-grid {
  display: grid;
  gap: 56px;
  align-items: center;
}

@media (min-width: 960px) {
  .hero { padding-top: 88px; }
  .hero-grid { grid-template-columns: 1.08fr 1fr; gap: 64px; }
}

.eyebrow {
  margin: 0 0 22px;
  font-family: var(--vp-font-family-mono);
  font-size: 13px;
  color: var(--bs-ink);
}

.eyebrow .caret { color: var(--bs-amber); margin-right: 4px; }
.eyebrow .dim { color: var(--bs-ink-3); }

.hero-title {
  margin: 0;
  font-size: clamp(2.4rem, 5.4vw, 3.9rem);
  line-height: 1.04;
  font-weight: 600;
  letter-spacing: -0.035em;
  color: var(--bs-ink);
  text-wrap: balance;
}

.hero-tagline {
  margin: 22px 0 0;
  font-family: var(--vp-font-family-mono);
  font-size: clamp(1rem, 1.5vw, 1.12rem);
  line-height: 1.55;
  color: var(--bs-amber);
  max-width: 34em;
}

.hero-body {
  margin: 18px 0 0;
  font-size: 1.05rem;
  max-width: 36em;
}

.actions {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  margin-top: 32px;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 11px 20px;
  border: 1px solid var(--bs-ink);
  border-radius: 3px;
  font-weight: 600;
  font-size: 15px;
  text-decoration: none;
  transition: transform 0.12s, box-shadow 0.12s, background 0.12s;
}

.btn.primary {
  background: var(--bs-ink);
  color: var(--bs-paper);
  box-shadow: 3px 3px 0 var(--bs-amber);
}

.btn.secondary {
  background: var(--bs-paper);
  color: var(--bs-ink);
  box-shadow: 3px 3px 0 var(--bs-rule-strong);
}

.dark .btn.primary {
  background: var(--bs-amber);
  border-color: var(--bs-amber);
  color: #15130f;
  box-shadow: 3px 3px 0 #5c432b;
}

.dark .btn.secondary {
  border-color: var(--bs-rule-strong);
  box-shadow: 3px 3px 0 #0b0a08;
}

.btn:hover { transform: translate(-1px, -1px); }
.btn.primary:hover { box-shadow: 4px 4px 0 var(--bs-amber); }
.dark .btn.primary:hover { box-shadow: 4px 4px 0 #5c432b; }
.btn:active { transform: translate(2px, 2px); box-shadow: 1px 1px 0 var(--bs-rule-strong); }

.hero-sub {
  margin: 20px 0 0;
  font-size: 15px;
}

.hero-sub a { text-decoration: none; font-weight: 500; }
.hero-sub a:hover { text-decoration: underline; text-underline-offset: 4px; }

.facts {
  display: flex;
  flex-wrap: wrap;
  gap: 0;
  margin: 28px 0 0;
  padding: 0;
  list-style: none;
  font-family: var(--vp-font-family-mono);
  font-size: 12.5px;
  color: var(--bs-ink-3);
}

.facts li:not(:last-child)::after {
  content: "·";
  margin: 0 10px;
  color: var(--bs-rule-strong);
}

.hero-art {
  position: relative;
  max-width: 520px;
  width: 100%;
  justify-self: center;
}

.art-frame {
  width: 86%;
  margin-left: auto;
  filter: drop-shadow(6px 6px 0 rgba(41, 36, 29, 0.10));
}

.dark .art-frame { filter: drop-shadow(6px 6px 0 rgba(0, 0, 0, 0.45)); }

.hero-term {
  position: relative;
  margin: -28px 0 0 !important;
  width: 92%;
  z-index: 1;
}

.prototype-note {
  margin-top: 56px;
  margin-bottom: 0;
  padding-top: 18px !important;
  padding-bottom: 22px !important;
  font-size: 13.5px;
  color: var(--bs-ink-3);
  border-top: 1px dashed var(--bs-rule-strong);
  max-width: var(--wrap);
}

.prototype-note strong { color: var(--bs-brick); font-weight: 600; }

@media (min-width: 768px) {
  .prototype-note { padding-left: 40px; padding-right: 40px; }
}

/* --- Windows ------------------------------------------------------------ */

.window {
  margin: 0;
  border: 1px solid var(--bs-ink);
  border-radius: 3px;
  background: var(--bs-window-bg);
  box-shadow: 5px 5px 0 rgba(41, 36, 29, 0.14);
  overflow: hidden;
}

.dark .window {
  border-color: var(--bs-rule-strong);
  box-shadow: 5px 5px 0 rgba(0, 0, 0, 0.5);
}

.window-bar {
  position: relative;
  display: flex;
  align-items: center;
  justify-content: center;
  height: 24px;
  border-bottom: 1px solid var(--bs-ink);
  background:
    repeating-linear-gradient(to bottom, var(--bs-window-bar) 0 2px, var(--bs-stripe) 2px 3px) 0 5px / 100% 13px no-repeat,
    var(--bs-window-bar);
}

.dark .window-bar { border-bottom-color: var(--bs-rule-strong); }

.close-box {
  position: absolute;
  left: 10px;
  top: 5px;
  width: 13px;
  height: 13px;
  border: 1px solid var(--bs-ink-2);
  background: var(--bs-window-bar);
}

.window-title {
  padding: 0 10px;
  background: var(--bs-window-bar);
  font-family: var(--vp-font-family-mono);
  font-size: 11.5px;
  font-weight: 500;
  color: var(--bs-ink-2);
}

.window-body {
  margin: 0;
  padding: 16px 18px 18px;
  overflow-x: auto;
  font-family: var(--vp-font-family-mono);
  font-size: 12.5px;
  line-height: 1.7;
  color: var(--bs-ink);
}

.window-body code {
  padding: 0;
  border: 0;
  background: none;
  font-size: inherit;
}

.window-body .c { color: var(--bs-ink-3); font-style: italic; }
.window-body .p { color: var(--bs-sage); font-weight: 500; }
.window-body .o { color: var(--bs-amber); }
.window-body .d { color: var(--bs-sage); font-weight: 500; }

.cursor {
  display: inline-block;
  width: 0.6em;
  height: 1.1em;
  vertical-align: -0.2em;
  background: var(--bs-amber);
  animation: blink 1.1s steps(1) infinite;
}

@keyframes blink { 50% { opacity: 0; } }

/* --- Problem ------------------------------------------------------------ */

.problem {
  display: grid;
  gap: 32px 72px;
}

@media (min-width: 960px) {
  .problem { grid-template-columns: 1fr 1.1fr; }
  .problem-answer { grid-column: 1 / -1; }
}

.situations {
  margin: 0;
  padding: 0;
  list-style: none;
  border-top: 1px solid var(--bs-rule-strong);
}

.situations li {
  display: grid;
  grid-template-columns: 40px 1fr;
  gap: 8px;
  padding: 16px 0;
  border-bottom: 1px solid var(--bs-rule);
  color: var(--bs-ink);
  line-height: 1.6;
}

.situations .num {
  font-family: var(--vp-font-family-mono);
  font-size: 12px;
  color: var(--bs-ink-3);
  padding-top: 3px;
}

.problem-answer {
  margin: 8px 0 0;
  padding: 24px 28px;
  font-size: 1.1rem;
  background: var(--bs-paper-2);
  border: 1px solid var(--bs-rule);
  border-left: 3px solid var(--bs-amber);
}

.problem-answer strong { color: var(--bs-ink); font-weight: 600; }

/* --- Ideas -------------------------------------------------------------- */

.ideas {
  display: grid;
  gap: 40px;
}

@media (min-width: 768px) {
  .ideas { grid-template-columns: repeat(3, 1fr); gap: 36px; }
}

.idea {
  padding-top: 28px;
  border-top: 2px solid var(--bs-ink);
}

.dark .idea { border-top-color: var(--bs-rule-strong); }

.idea h3 {
  margin: 0 0 12px;
  font-size: 1.2rem;
  line-height: 1.35;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--bs-ink);
}

.idea p:last-child { margin: 0; }

/* --- Loop --------------------------------------------------------------- */

.loop {
  display: grid;
  gap: 56px;
  align-items: start;
}

@media (min-width: 960px) {
  .loop { grid-template-columns: 1fr 1fr; gap: 72px; }
}

.loop h2 { margin-bottom: 18px; }

.requests {
  display: grid;
  gap: 10px;
  margin: 18px 0 18px;
}

.request {
  margin: 0;
  display: flex;
  align-items: baseline;
  gap: 14px;
  padding: 12px 16px;
  border: 1px solid var(--bs-rule);
  border-left: 3px solid var(--bs-amber);
  background: var(--bs-paper-2);
  color: var(--bs-ink);
}

.request .ctx {
  flex: none;
  font-family: var(--vp-font-family-mono);
  font-size: 11px;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--bs-ink-3);
}

.retrieval {
  border: 1px solid var(--bs-rule-strong);
  background: var(--bs-paper-2);
  border-radius: 3px;
}

.retrieval-head {
  padding: 24px 26px 8px;
}

.retrieval-head p:last-child { margin: 0; }

.retrieval ol {
  margin: 12px 0 0;
  padding: 0;
  list-style: none;
}

.retrieval li {
  display: grid;
  grid-template-columns: 28px 1fr;
  gap: 14px;
  padding: 14px 26px;
  border-top: 1px dotted var(--bs-rule-strong);
}

.retrieval li p { margin: 4px 0 0; font-size: 14.5px; }

.step {
  display: grid;
  place-items: center;
  width: 24px;
  height: 24px;
  border: 1px solid var(--bs-ink-2);
  font-family: var(--vp-font-family-mono);
  font-size: 12px;
  color: var(--bs-ink);
}

.retrieval-foot {
  margin: 0;
  padding: 14px 26px 18px;
  border-top: 1px solid var(--bs-rule-strong);
  font-size: 14.5px;
  font-style: italic;
}

/* --- Folder ------------------------------------------------------------- */

.folder {
  display: grid;
  gap: 48px;
}

@media (min-width: 960px) {
  .folder { grid-template-columns: 1fr 1fr; gap: 72px; }
}

.folder h2 { margin-bottom: 16px; }
.folder h2 em { font-style: italic; color: var(--bs-amber); }

.tree { margin-top: 28px; max-width: 460px; }

.adds {
  margin: 0;
  border-top: 1px solid var(--bs-rule-strong);
}

.add {
  padding: 18px 0;
  border-bottom: 1px solid var(--bs-rule);
}

.add dt {
  font-weight: 600;
  color: var(--bs-ink);
}

.add dt::before {
  content: '+ ';
  font-family: var(--vp-font-family-mono);
  color: var(--bs-sage);
}

.add dd {
  margin: 4px 0 0;
  color: var(--bs-ink-2);
  line-height: 1.65;
  font-size: 15px;
}

/* --- Recipes panel ------------------------------------------------------ */

.panel {
  display: grid;
  gap: 40px;
  padding: 36px 28px;
  border: 1px solid var(--bs-ink);
  border-radius: 4px;
  background: var(--bs-paper-2);
  box-shadow: 6px 6px 0 var(--bs-rule-strong);
}

.dark .panel {
  border-color: var(--bs-rule-strong);
  box-shadow: 6px 6px 0 #0b0a08;
}

@media (min-width: 960px) {
  .panel { grid-template-columns: 1fr 1.1fr; gap: 64px; padding: 48px; }
}

.panel h2 { margin-bottom: 16px; }

.recipe-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.recipe-list li + li { border-top: 1px solid var(--bs-rule-strong); }

.recipe-list a {
  display: block;
  padding: 16px 0;
  text-decoration: none;
}

.recipe-list li:first-child a { padding-top: 0; }
.recipe-list li:last-child a { padding-bottom: 0; }

.recipe-title {
  display: block;
  font-weight: 600;
  color: var(--bs-ink);
  transition: color 0.12s;
}

.recipe-body {
  display: block;
  margin-top: 4px;
  font-size: 14.5px;
  color: var(--bs-ink-2);
}

.recipe-list a:hover .recipe-title { color: var(--bs-amber); }

/* --- Next --------------------------------------------------------------- */

.next {
  display: grid;
  gap: 0;
  grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
  border-top: 1px solid var(--bs-rule-strong);
  border-bottom: 1px solid var(--bs-rule-strong);
  padding-top: 0;
  padding-bottom: 0;
}

.next a {
  position: relative;
  display: block;
  padding: 22px 36px 22px 0;
  font-weight: 600;
  color: var(--bs-ink);
  text-decoration: none;
}

.next a .label { display: block; margin-bottom: 6px; font-size: 11px; color: var(--bs-ink-3); }
.next a .arrow { position: absolute; right: 20px; bottom: 22px; color: var(--bs-amber); transition: transform 0.12s; }
.next a:hover { color: var(--bs-amber); }
.next a:hover .arrow { transform: translateX(3px); }

@media (max-width: 639px) {
  .section { margin-top: 80px; }
  .hero-term { width: 100%; margin-top: 16px !important; }
  .art-frame { width: 100%; }
}
</style>
