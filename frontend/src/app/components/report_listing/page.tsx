"use client";

import React, { useEffect, useState } from "react";

/// A representation of the `libghr::report::Report`. See it for more info. 
interface Report {
    os: OsInfo,
}

interface OsInfo {
    name: string,
    version: string,
    architecture: string,
}


const ReportListing: React.FC = () => {
    const [reports, set_reports] = useState<Report[]>([]);
    // const [loading, setLoading] = useState<boolean>(true);

    useEffect(() => {
        const get_reports = async () => {
            const resp = await fetch("http://localhost:8080/reports");

            const json = await resp.json();
            console.log(json);
            set_reports(json);
        }

        get_reports();
    }, []); // apparently `[]` stops it from doing this... way too much.

    return (
        <div className="report_listing">
            <h2>Reports</h2>
            <ul>
                {reports.map((report) => (
                    <li key={report.os.name}>
                        <p>{report.os.name}, {report.os.version} ({report.os.architecture})</p>
                    </li>
                ))}
            </ul>
        </div>
    )
};

export default ReportListing;