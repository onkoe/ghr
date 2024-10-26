import type { ComponentDescription } from "../../../../shared/bindings/ComponentDescription";
import type { ComponentInfo } from "../../../../shared/bindings/ComponentInfo";
import type { CpuDescription } from "../../../../shared/bindings/CpuDescription";
import type { Report } from "../../../../shared/bindings/Report";
import type { WrappedReportTs } from "../../../../shared/bindings/WrappedReportTs";
import { CmpType, icon } from "../components/cmp_icon/page";
import {
	CmpTile,
	type CmpTileProps,
} from "../components/report_listing/tile/mod";

// we'll start picking apart the report. let's start with the computer's name
export default function ReportPage(wrapped_report: WrappedReportTs) {
	const report: Report = wrapped_report.report;
	const computer_name = report.os.name;

	const cpu_components = [];

	for (const [general, cpu] of cpu_descriptions(report)) {
		const value_string: string = `max is ${
			cpu.clock_speed.max || 0
		}. min is ${cpu.clock_speed.min || 0}`;

		cpu_components.push(
			<CmpTile
				name={general.id || "no id found"}
				cmp={CmpType.Cpu}
				value={value_string}
			/>,
		);
	}

	return (
		<div>
			<p>{wrapped_report.id}</p>
			<p>{computer_name}</p>
			{cpu_components}
		</div>
	);
}

/**
 *
 * @param report A `Report` containing info about the system.
 * @returns A list of `CpuDescription`s.
 */
function cpu_descriptions(report: Report): [ComponentInfo, CpuDescription][] {
	const processors: [ComponentInfo, CpuDescription][] = [];

	for (const cmp of report.components) {
		const desc: ComponentDescription = cmp.desc;

		if (typeof desc === "object" && "CpuDescription" in desc) {
			const cpu_desc: CpuDescription = desc.CpuDescription;
			processors.push([cmp, cpu_desc]);
		}
	}

	return processors;
}
