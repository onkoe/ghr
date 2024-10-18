"use client";

import styles from './Button.module.scss';

interface ButtonProps {
    label: string;
    onClick: () => void;
    variant?: 'primary' | 'secondary';
}

const Button: React.FC<ButtonProps> = ({ label, onClick, variant = 'primary' }) => {
    return (
        <button onClick={onClick} className={styles[variant]}>
            {label}
        </button>
    );
};

export default Button;