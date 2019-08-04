fn f() {
    NothingInMe {};
    Point { x: 10.0, y: 20.0 };
    TuplePoint { 0: 10.0, 1: 20.0 };
    Point3d { y: 0, z: 10, ..base };
}
