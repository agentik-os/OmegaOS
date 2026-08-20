# Workflow: Signal watchlist

**Produces:** a watchlist entry that can actually be swept: a named behaviour
that would have to change, independent sources with their interest declared, a
cadence, and a next review date.

## Trigger

A domain has started mattering to a decision (timing, positioning, a build or a
bet), and the current method for noticing change in it is "I read things".

## Steps

1. **Name the decision the watch serves.** Who will act on it, and what they
   would do differently if the trend were confirmed. A watch with no decision
   behind it produces reading material, and it is refused here rather than
   discovered as useless in six months.
2. **Write the behaviour that would have to change.** State it so somebody could
   go and look for it: a purchase, a deployment, a hire, a migration, a
   cancellation, a filing, a price move, a registration. Reject attention
   measures at this step (coverage, search volume, posts, conference agendas):
   they may be captured later as attention, but they may not define the watch.
3. **State the current baseline, with its date.** What is true today, as far as
   you can tell, and when you established that. Without a baseline there is
   nothing for a rate to be measured against later.
4. **Choose sources.** At least two that do not depend on each other, and at
   least one that is not selling into the trend. For each source record: what it
   observes, its interest label (disinterested, participant, vendor selling into
   the trend, funded by a participant), its access constraint (free, paid, rate
   limited, terms restricted) and its typical lag between an event happening and
   the source reporting it.
5. **Check for source collapse before committing.** If several candidate sources
   would all be restating one origin, they count as one. Say so now, and find a
   genuinely independent second.
6. **Route access approvals.** Anything paid, rate limited, forbidden by terms,
   or run as a standing automated watcher goes to the human approval boundary
   now, before the first sweep, not when the bill arrives.
7. **Set the cadence and the next review date.** Match the cadence to the lag of
   the slowest source: sweeping weekly a source that reports quarterly produces
   noise and a false sense of coverage.
8. **Seed from what you already have.** Pull dated material on this domain from
   Librarian {OS} (`librarian.source.indexed`) and any relevant background from
   Research {OS} (`research.evidence.compiled`), and capture it as signals with
   their real observation dates rather than today's.
9. **Record the watch** to canonical state and set its review date.

## Completion test

- The watch names one decision and one observable behaviour, not a topic.
- The behaviour statement is a behaviour measure, not an attention measure.
- A baseline is written down with the date it was established.
- At least two independent sources are listed, and at least one is labelled
  disinterested.
- Every source carries an interest label, an access constraint and a reporting
  lag.
- The cadence is no faster than the slowest source can support.
- Every paid, restricted or automated source has an approval status, and none is
  in use without one.
- A next review date exists and is in the calendar, not in the prose.
