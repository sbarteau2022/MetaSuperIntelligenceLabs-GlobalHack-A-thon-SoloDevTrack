import './App.css'

function App() {
  return (
    <>
      <header className="site-header">
        <a className="wordmark" href="#top">
          Atoma <span>Labs</span>
        </a>
        <nav aria-label="Primary">
          <a href="#top">Home</a>
          <a href="#overview">About</a>
          <a href="#insights">Updates</a>
          <a href="#insights">Blog</a>
        </nav>
        <a className="btn btn-primary" href="#cta">
          GET STARTED
        </a>
      </header>

      <main id="top">
        <section className="hero">
          <p className="kicker">Built for engineering teams</p>
          <h1>One mind. Every engineering job.</h1>
          <p className="lede">
            Atoma connects your research, code, and knowledge into one judged
            mind map. It retrieves, checks, and acts on real evidence — with
            you in control of every step.
          </p>
          <div className="hero-actions">
            <a className="btn btn-primary" href="#cta">
              GET STARTED
            </a>
            <span
              className="btn btn-ghost is-disabled"
              aria-disabled="true"
              title="Video coming soon"
            >
              PLAY VIDEO
            </span>
          </div>
          <div className="layer-grid">
            <article>
              <p className="card-no">01</p>
              <h2>Connect Layer</h2>
              <p>
                Plugs into your docs, code, and research. Captures every source
                with provenance. Nothing enters the map unjudged.
              </p>
            </article>
            <article>
              <p className="card-no">02</p>
              <h2>Action Layer</h2>
              <p>
                Runs real tool calls and system updates. Every action is
                receipted. Failures show their cause, not just an error.
              </p>
            </article>
            <article>
              <p className="card-no">03</p>
              <h2>Control Layer</h2>
              <p>
                Policies, approvals, and audit gates. Sensitive actions wait for
                a human. The automaton watches, it does not freelance.
              </p>
            </article>
            <article>
              <p className="card-no">04</p>
              <h2>Context Layer</h2>
              <p>
                Structured memory from one map. Recalled atoms carry their
                source. Context improves with use, not with hope.
              </p>
            </article>
          </div>
        </section>

        <section className="trust-strip" aria-label="Trusted by">
          <p className="kicker">Trusted by</p>
          <ul>
            <li>
              <a
                href="https://about.meta.com/"
                target="_blank"
                rel="noreferrer"
              >
                <img
                  src="/logos/meta-superintelligence.svg"
                  alt="Meta"
                  height="30"
                />
                <span className="meta-name">Meta Superintelligence</span>
              </a>
            </li>
            <li>
              <a
                href="https://www.baseten.co/"
                target="_blank"
                rel="noreferrer"
                aria-label="Baseten"
              >
                <img src="/logos/baseten.svg" alt="Baseten" height="30" />
              </a>
            </li>
            <li>
              <a
                href="https://cerebralvalley.ai/"
                target="_blank"
                rel="noreferrer"
              >
                <img
                  className="cv-mark"
                  src="/logos/cerebral-valley.png"
                  alt="Cerebral Valley"
                  width="34"
                  height="34"
                />
                <span>Cerebral Valley</span>
              </a>
            </li>
          </ul>
        </section>

        <nav className="anchor-nav" aria-label="Sections">
          <a href="#capabilities">Capabilities</a>
          <a href="#numbers">Numbers</a>
          <a href="#features">Features</a>
          <a href="#integrations">Integrations</a>
          <a href="#pricing">Pricing</a>
          <a href="#insights">Insights</a>
          <a href="#faq">FAQ</a>
        </nav>

        <section id="overview" className="block">
          <p className="kicker">Overview</p>
          <p className="overview-text">
            Atoma is a shared, judged mind for engineering work — one hyperbolic
            map, built from many models, that retrieves atoms, verifies them,
            and acts on them. It keeps the team in control while the repetitive
            work gets done once, properly.
          </p>
          <ol className="workflow">
            <li>
              <span>01</span>
              <div>
                <h2>Source connect</h2>
                <p>Connect sources with provenance attached.</p>
              </div>
            </li>
            <li>
              <span>02</span>
              <div>
                <h2>Action runner</h2>
                <p>Execute updates, tasks, and notifications.</p>
              </div>
            </li>
            <li>
              <span>03</span>
              <div>
                <h2>Approval gate</h2>
                <p>Require approval for sensitive actions.</p>
              </div>
            </li>
            <li>
              <span>04</span>
              <div>
                <h2>Audit + insights</h2>
                <p>Track changes and spot patterns.</p>
              </div>
            </li>
          </ol>
        </section>

        <section id="capabilities" className="block">
          <p className="kicker">Capabilities</p>
          <h2 className="section-title">Four layers that map, judge, and execute.</h2>
          <p className="section-sub">
            Atoma connects your stack, applies policies and approvals, and keeps
            automation safe. One map, one governor, every action receipted.
          </p>
          <ul className="cap-list">
            <li>
              <h3>Connect</h3>
              <p>Plug into docs, code, and research in minutes.</p>
            </li>
            <li>
              <h3>Resolve</h3>
              <p>Turn each request into a completed, evidenced outcome.</p>
            </li>
            <li>
              <h3>Control</h3>
              <p>Set policies, approvals, and routing for safety.</p>
            </li>
            <li>
              <h3>Analyze</h3>
              <p>See what works, what is risky, and what to automate next.</p>
            </li>
          </ul>
        </section>

        <section id="numbers" className="block">
          <p className="kicker">Numbers</p>
          <div className="numbers-panel">
            <p className="numbers-label">Unified map</p>
            <p className="numbers-figures">
              1 map <span>·</span> 12 basins <span>·</span> 5 models distilled
            </p>
            <p className="numbers-qual">With policies + approvals.</p>
          </div>
        </section>

        <section id="features" className="block">
          <p className="kicker">Features</p>
          <h2 className="section-title">Core systems that power every run.</h2>
          <p className="section-sub">
            Real-time retrieval, judging, and action across your work — so speed
            never costs you the evidence.
          </p>
          <ol className="feature-rows">
            <li>
              <span>001</span>
              <div>
                <h3>Context Assembly</h3>
                <p>
                  Unify docs, code, and research into one structured,
                  source-grounded view.
                </p>
              </div>
            </li>
            <li>
              <span>002</span>
              <div>
                <h3>Policy &amp; Approvals</h3>
                <p>
                  Configurable rules, review of sensitive actions, and alignment
                  with your standards.
                </p>
              </div>
            </li>
            <li>
              <span>003</span>
              <div>
                <h3>Action Execution</h3>
                <p>
                  Trigger real updates, modify records, and complete system
                  actions with full audit visibility.
                </p>
              </div>
            </li>
          </ol>
        </section>

        <section id="integrations" className="block">
          <p className="kicker">Integrations</p>
          <h2 className="section-title">
            Connect Atoma to the tools you already use.
          </h2>
          <p className="section-sub">
            Direct integration with your core stack, so Atoma can retrieve real
            data and trigger real workflows.
          </p>
          <ul className="tab-row" aria-label="Integration areas">
            <li>Documentation</li>
            <li>Communication</li>
            <li>Support &amp; helpdesk</li>
            <li>CRM &amp; revenue</li>
          </ul>
          <ul className="badge-row">
            <li>Bidirectional</li>
            <li>Secured by default</li>
            <li>Context-aware</li>
          </ul>
        </section>

        <section id="pricing" className="block">
          <p className="kicker">Pricing</p>
          <h2 className="section-title">Simple plans that scale with you.</h2>
          <p className="section-sub">
            Free for contributors. Paid for consume-only use. Pricing grows from
            first runs to enterprise scale.
          </p>
          <div className="tier-grid">
            <article>
              <h3>Contributor</h3>
              <p className="price">Free</p>
              <p>For builders who contribute to the shared mind.</p>
              <a className="btn btn-ghost" href="#cta">
                GET STARTED
              </a>
            </article>
            <article>
              <h3>Team</h3>
              <p className="price">Soon</p>
              <p>For teams running judged agentic work daily.</p>
              <a className="btn btn-ghost" href="#cta">
                GET STARTED
              </a>
            </article>
            <article>
              <h3>Enterprise</h3>
              <p className="price">Custom</p>
              <p>For orgs needing advanced security and scale.</p>
              <a className="btn btn-ghost" href="#cta">
                TALK TO SALES
              </a>
            </article>
          </div>
        </section>

        <section id="insights" className="block">
          <p className="kicker">Blog</p>
          <h2 className="section-title">
            Articles, notes, updates, and case studies.
          </h2>
          <p className="section-sub">
            Insights on judged minds, automation design, and systems built to
            last.
          </p>
          <ul className="insight-list">
            <li>
              <p className="insight-cat">Updates</p>
              <h3>Why retrieval alone fails, and what a judged map changes</h3>
            </li>
            <li>
              <p className="insight-cat">Updates</p>
              <h3>From prompt to mechanism: building automation you can audit</h3>
            </li>
            <li>
              <p className="insight-cat">Updates</p>
              <h3>The hidden cost of context loss in long engineering runs</h3>
            </li>
          </ul>
        </section>

        <section id="faq" className="block">
          <p className="kicker">FAQ</p>
          <h2 className="section-title">Frequently asked questions.</h2>
          <div className="faq-list">
            <details open>
              <summary>Do I have to train the model?</summary>
              <p>
                No. Atoma retrieves and judges from your map and sources;
                training is not the mechanism here.
              </p>
            </details>
            <details>
              <summary>What control do I keep?</summary>
              <p>
                Policies and approvals decide what runs automatically and what
                waits for you.
              </p>
            </details>
            <details>
              <summary>What is Atoma, in one line?</summary>
              <p>
                A shared, judged mind for engineering work, distributed as a
                brain that attaches to your setup.
              </p>
            </details>
          </div>
          <aside className="help-panel">
            <p className="kicker">Still have questions? We are here to help</p>
            <p>
              For complex workflows or custom integrations, talk to us.
            </p>
            <a className="btn btn-primary" href="#cta">
              CONTACT US
            </a>
          </aside>
        </section>

        <section id="cta" className="block cta">
          <p className="kicker">Ready to build with Atoma</p>
          <h2 className="section-title">Build once. Prove it. Ship it.</h2>
          <p className="section-sub">
            Connect your stack, set your guardrails, and let Atoma handle the
            repetitive work while you stay in control.
          </p>
          <a className="btn btn-primary" href="#pricing">
            GET STARTED
          </a>
        </section>
      </main>

      <footer>
        <p className="wordmark">Atoma</p>
        <p className="foot-desc">
          Atoma Labs builds Ares — SpecOps, a judged engineering mind that
          retrieves, verifies, and acts — with receipts, and with you in
          control.
        </p>
        <nav aria-label="Pages">
          <a href="#top">Homepage</a>
          <a href="#overview">About</a>
          <a href="#insights">Updates</a>
          <a href="#cta">Waitlist</a>
          <a href="#insights">Blog</a>
        </nav>
        <p className="bottom-bar">
          © Atoma Labs, 2026. All rights reserved. Terms of use. Privacy
          Policy.
        </p>
      </footer>
    </>
  )
}

export default App
