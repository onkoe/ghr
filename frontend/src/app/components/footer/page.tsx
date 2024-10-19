import styles from "./footer.module.scss";

import { Github } from "lucide-react";
import { Button } from "@/components/ui/button";
import Link from "next/link";

const Footer: React.FC = () => {
    return (
        <footer className={styles.footer}>
            <p>© GHR Contributors 2024</p>
            <div className="actions">
                <Button variant="outline" size="icon" asChild>
                    <Link href="https://github.com/onkoe/ghr">
                        <Github className="h-4 w-4" />
                    </Link>
                </Button>
            </div>
        </footer>
    );
};

export default Footer;
