import styles from "./page.module.css";
import NavBar from "./navbar/page";
// import Button from "./m3/button/page"
import ReportListing from "./components/report_listing/page";

export default function Home() {
  return (
    <div className={styles.page}>
      <NavBar />

      <main className={styles.main}>
        {/* <Button label="hi" onClick={() => console.log("hi clicked")} /> */}
        <p>{hello_from_actix()}</p>

        <ReportListing />

      </main>
      <footer className={styles.footer}>
        <p>footer</p>
      </footer>
    </div>
  );
}

async function hello_from_actix(): Promise<string> {
  const thing: string = await fetch("http://localhost:8080").then(response => response.text());
  return thing;
}