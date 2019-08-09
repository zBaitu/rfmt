#[cfg(test)]
mod tests {
    #[test]
    fn handles_mid_demangling() {
        assert_eq!(
            crate::demangle_line("        lea     rax, [rip + _ZN55_$LT$$RF$$u27$a$u20$T$u20$as$u20$core..fmt..Display$GT$3fmt17h510ed05e72307174E]"),
                "        lea     rax, [rip + <&\'a T as core::fmt::Display>::fmt]");
    }
}
