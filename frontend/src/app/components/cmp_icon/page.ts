import {
    mdiCpu64Bit,
    mdiExpansionCard,
    mdiMemory,
    mdiTapeDrive,
    mdiExpansionCardVariant,
    mdiUsbPort,
    mdiHelpRhombus,
} from "@mdi/js";

/**
 * An enumeration over possible icons for device (component/"cmp") types.
 */
export const CmpType = {
    Cpu: "cpu",
    Gpu: "gpu",
    Ram: "ram",
    Storage: "storage",
    Pci: "pci",
    Usb: "usb",
} as const;

/**
 * An export for `CmpIcon`, an enumeration over icons for cmps.
 */
export type CmpType = (typeof CmpType)[keyof typeof CmpType];

import { match } from "ts-pattern";

/**
 *
 * @param cmp A type of computer component.
 * @returns A path to the icon the UI should display.
 */
export function icon(cmp: CmpType): string {
    const icon_class: string = match(cmp)
        .with(CmpType.Cpu, () => mdiCpu64Bit)
        .with(CmpType.Gpu, () => mdiExpansionCard)
        .with(CmpType.Ram, () => mdiMemory)
        .with(CmpType.Storage, () => mdiTapeDrive)
        .with(CmpType.Pci, () => mdiExpansionCardVariant)
        .with(CmpType.Usb, () => mdiUsbPort)
        .otherwise(() => mdiHelpRhombus);
        
    return icon_class;
}
