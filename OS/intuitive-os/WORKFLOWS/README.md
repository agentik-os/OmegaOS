# Intuitive {OS}: Workflows

Repeatable processes this OS runs. Each workflow is one file, named for what it
produces, and states its trigger, steps, and completion test.

| Workflow | Trigger | Produces |
|---|---|---|
| [`signal-capture.md`](signal-capture.md) | a pull, a doubt or a certainty arrives while the outcome is still unknown, or Decision {OS} asks for a gut input while framing a call | one immutable signal record: the falsifiable claim, domain, confidence, base rate, disconfirmer, resolution condition and resolution date |
| [`monthly-calibration-report.md`](monthly-calibration-report.md) | a month closes, or a domain's unresolvable rate crosses 30 percent | the calibration report: per domain count, hit rate, Brier, skill, tier, weight and staleness, plus closed overdue signals, tier changes and capture-quality defects |

The two are one loop with a long delay between its halves. Capture writes a
prediction that cannot be edited; the monthly report is where that prediction
finally costs or earns something. A month of captures with no report produces
no weight, and a report with no captures behind it produces only staleness.
