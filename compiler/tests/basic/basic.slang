/**
    Foo
*/
fragment credit-app {
    read;

    supplier {
        read;
        supplier-app { #credit-app; }
    }

    customer {
        read;
        customer-app { #credit-app; }
    }
}

@user {
    personnel:is-admin {
        read.basic;
        write;

        supplier-app { #credit-app; }
    }
}
