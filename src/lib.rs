extern crate proc_macro2;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;
#[cfg_attr(test, macro_use)]
extern crate syn;
#[macro_use]
extern crate synstructure;

use proc_macro2::TokenStream;

decl_derive!([Arbitrary] => arbitrary_derive);

fn arbitrary_derive(s: synstructure::Structure) -> TokenStream {
    let mut variants = s
        .variants()
        .iter()
        .map(|variant| variant.construct(|_, _| quote!(Arbitrary::arbitrary(g))));

    let (g, body) = match s.variants().len() {
        // zero-variant enum
        0 => panic!("Cannot derive `Arbitrary` for an enum with no variants."),

        // struct or single-variant enum
        1 => {
            let body = variants.next().unwrap();

            let g = if let syn::Fields::Unit = s.variants()[0].ast().fields {
                quote!(_g)
            } else {
                quote!(g)
            };

            (g, body)
        }

        // multiple-variant enum
        count => {
            let variants = variants.enumerate().map(|(i, variant)| quote!(#i => #variant));

            let body = quote! {
                match g.gen_range(0, #count) {
                    #(#variants,)*
                    _ => unreachable!()
                }
            };

            (quote!(g), body)
        }
    };

    s.gen_impl(quote! {
        extern crate quickcheck;
        extern crate rand;

        use quickcheck::{Arbitrary, Gen};

        #[allow(unused_imports)]
        use rand::Rng;

        gen impl Arbitrary for @Self {
            fn arbitrary<G: Gen>(#g: &mut G) -> Self {
                #body
            }
        }
    })
}

#[test]
fn test_arbitrary_unit_struct() {
    test_derive! {
        arbitrary_derive {
            #[derive(Clone)]
            struct ArbitraryTest;
        }
        expands to {
            #[allow(non_upper_case_globals)]
            const _DERIVE_Arbitrary_FOR_ArbitraryTest: () = {
                extern crate quickcheck;
                extern crate rand;

                use quickcheck::{Arbitrary, Gen};

                #[allow(unused_imports)]
                use rand::Rng;

                impl Arbitrary for ArbitraryTest {
                    fn arbitrary<G: Gen>(_g: &mut G) -> Self {
                        ArbitraryTest
                    }
                }
            };
        }
    }
}

#[test]
fn test_arbitrary_struct() {
    test_derive! {
        arbitrary_derive {
            #[derive(Clone)]
            struct ArbitraryTest(u8, bool);
        }
        expands to {
            #[allow(non_upper_case_globals)]
            const _DERIVE_Arbitrary_FOR_ArbitraryTest: () = {
                extern crate quickcheck;
                extern crate rand;

                use quickcheck::{Arbitrary, Gen};

                #[allow(unused_imports)]
                use rand::Rng;

                impl Arbitrary for ArbitraryTest {
                    fn arbitrary<G: Gen>(g: &mut G) -> Self {
                        ArbitraryTest(Arbitrary::arbitrary(g),
                                      Arbitrary::arbitrary(g), )
                    }
                }
            };
        }
    }
}

#[test]
#[should_panic(expected = "Cannot derive `Arbitrary` for an enum with no variants.")]
fn test_arbitrary_zero_variant_enum() {
    let input = parse_quote! {
        #[derive(Clone)]
        enum ArbitraryTest {}
    };

    arbitrary_derive(synstructure::Structure::new(&input));
}

#[test]
fn test_arbitrary_enum() {
    test_derive! {
        arbitrary_derive {
            #[derive(Clone)]
            enum ArbitraryTest {
                A,
                B(usize, u32),
                C{ b: bool, d: (u16, u16) }
            }
        }
        expands to {
            #[allow(non_upper_case_globals)]
            const _DERIVE_Arbitrary_FOR_ArbitraryTest: () = {
                extern crate quickcheck;
                extern crate rand;

                use quickcheck::{Arbitrary, Gen};

                #[allow(unused_imports)]
                use rand::Rng;

                impl Arbitrary for ArbitraryTest {
                    fn arbitrary<G: Gen>(g: &mut G) -> Self {
                        match g.gen_range(0, 3usize) {
                            0usize => ArbitraryTest::A,
                            1usize => ArbitraryTest::B(Arbitrary::arbitrary(g),
                                                       Arbitrary::arbitrary(g),
                                                      ),
                            2usize => ArbitraryTest::C {
                                    b : Arbitrary::arbitrary(g),
                                    d : Arbitrary::arbitrary(g),
                                },
                            _ => unreachable!()
                        }
                    }
                }
            };
        }
    }
}
