import { promises as fs } from "node:fs";
import path from "node:path";
import { Whitepaper } from "../../../components/templates/Whitepaper";
import { AuditReport } from "../../../components/templates/AuditReport";
import { MarketingReport } from "../../../components/templates/MarketingReport";
import { GenericDoc } from "../../../components/templates/GenericDoc";
import {
  sampleWhitepaper,
  sampleAudit,
  sampleMarketing,
  sampleDoc
} from "../../../lib/samples";
import type { AnyData } from "../../../lib/schemas";

const SAMPLES: Record<string, AnyData> = {
  whitepaper: sampleWhitepaper,
  audit: sampleAudit,
  marketing: sampleMarketing,
  doc: sampleDoc
};

async function loadData(template: string, dataFile: string | null, demo: boolean): Promise<AnyData> {
  if (demo) return SAMPLES[template];
  if (dataFile) {
    const raw = await fs.readFile(path.resolve(dataFile), "utf-8");
    return JSON.parse(raw) as AnyData;
  }
  return SAMPLES[template];
}

export default async function RenderPage({
  params,
  searchParams
}: {
  params: Promise<{ template: string }>;
  searchParams: Promise<{ demo?: string; data?: string }>;
}) {
  const { template } = await params;
  const sp = await searchParams;
  const demo = sp.demo === "1" || sp.demo === "true";
  const dataFile = sp.data || null;
  const data = await loadData(template, dataFile, demo);

  if (data.template === "whitepaper") return <Whitepaper data={data} />;
  if (data.template === "audit") return <AuditReport data={data} />;
  if (data.template === "marketing") return <MarketingReport data={data} />;
  if (data.template === "doc") return <GenericDoc data={data} />;
  return <div style={{ padding: 40 }}>Unknown template: {template}</div>;
}

export const dynamic = "force-dynamic";
