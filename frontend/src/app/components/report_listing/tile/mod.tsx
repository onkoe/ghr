"use client";

import { type CmpType, icon } from "../../cmp_icon/page";
import Icon from "@mdi/react";
import styles from "./tile.module.scss";

export interface CmpTileProps {
    name: string;
    cmp: CmpType;
    value: string;
}

export const CmpTile = (props: CmpTileProps) => {
    const icon_path: string = icon(props.cmp);

    return (
        <div className={styles.cmp_tile}>
            <Icon path={icon_path} />
            <p className={styles.cmp_tile_name}>NAME: {props.name}</p>
            <p className={styles.cmp_tile_value}>VALUE: {props.value}</p>
        </div>
    );
};
