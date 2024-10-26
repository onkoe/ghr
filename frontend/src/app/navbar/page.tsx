import {
	NavigationMenu,
	NavigationMenuLink,
	NavigationMenuList,
	navigationMenuTriggerStyle,
} from "@/components/ui/navigation-menu";

import Link from "next/link";

export default function NavBar() {
	return (
		<NavigationMenu>
			<NavigationMenuList>
				<Link href="/" legacyBehavior passHref>
					<NavigationMenuLink className={navigationMenuTriggerStyle()}>
						Home
					</NavigationMenuLink>
				</Link>

				<Link href="/reports" legacyBehavior passHref>
					<NavigationMenuLink className={navigationMenuTriggerStyle()}>
						Reports
					</NavigationMenuLink>
				</Link>
			</NavigationMenuList>
		</NavigationMenu>
	);
}
