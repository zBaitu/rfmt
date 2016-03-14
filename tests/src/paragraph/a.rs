impl A {
    fn trans_ref_type(&mut self, lifetime: &Option<rst::Lifetime>, mut_type: &rst::MutTy) 
        -> RefType {
        RefType {
            lifetime: zopt::map_ref_mut(lifetime, |lifetime| self.trans_lifetime(lifetime)),
            is_mut: is_mut(mut_type.mutbl),
            ty: self.trans_type(&mut_type.ty),
        }
    }
}

