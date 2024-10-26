"use client";

import ReportPage from "@/app/report/page";
import type React from "react";
import { useEffect, useState } from "react";
import type { WrappedReportTs } from "../../../../../shared/bindings/WrappedReportTs";

const ReportListing: React.FC = () => {
	const [reports, set_reports] = useState<WrappedReportTs[]>([]);
	// const [loading, setLoading] = useState<boolean>(true);

	useEffect(() => {
		const get_reports = async () => {
			const resp = await fetch("http://localhost:8080/reports");

			const json = await resp.json();
			console.log(json);
			set_reports(json);
		};

		get_reports();
	}, []); // apparently `[]` stops it from doing this... way too much.

	return (
		<div className="report_listing">
			<h2>Reports</h2>
			<ul>
				{reports.map((report) => (
					<li key={report.id}>
						<ReportPage
							id={report.id}
							recv_time={report.recv_time}
							report={report.report}
						/>
					</li>
				))}
			</ul>
		</div>
	);
};

export default ReportListing;
